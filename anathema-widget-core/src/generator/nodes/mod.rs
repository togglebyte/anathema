use std::borrow::Cow;
use std::ops::DerefMut;
use std::rc::Rc;

use anathema_render::Size;
use anathema_values::{Collection, Context, NodeId, Path, Scope, ScopeValue, State};

use self::controlflow::{Else, If};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::expressions::Expression;
use crate::WidgetContainer;

mod controlflow;

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct LoopNode {
    pub(crate) body: Nodes,
    pub(crate) binding: Path,
    pub(crate) collection: Collection,
    pub(crate) value_index: usize,
}

impl LoopNode {
    fn scope(&mut self, scope: &mut Scope) {
        scope.scope_collection(
            self.binding.clone(),
            &self.collection,
            self.value_index,
        );
        self.body.reset();
        self.value_index += 1;
    }
}

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub(crate) kind: NodeKind,
}

impl Node {
    fn reset_cache(&mut self) {
        match &mut self.kind {
            NodeKind::Single(_, nodes) => nodes.reset_cache(),
            NodeKind::Loop(LoopNode { body, .. }) => body.reset_cache(),
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }
}

#[cfg(test)]
impl Node {
    pub(crate) fn single(&mut self) -> (&mut WidgetContainer, &mut Nodes) {
        match &mut self.kind {
            NodeKind::Single(inner, nodes) => (inner, nodes),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeKind {
    Single(WidgetContainer, Nodes),
    Loop(LoopNode),
    ControlFlow {
        if_node: If,
        elses: Vec<Else>,
        body: Nodes,
    },
}

#[derive(Debug)]
// TODO: possibly optimise this by making nodes optional on the node
pub struct Nodes {
    expressions: Rc<[Expression]>,
    pub inner: Vec<Node>,
    active_loop: Option<usize>,
    expr_index: usize,
    next_id: NodeId,
    cache_index: usize,
}

impl Nodes {
    pub fn update(&mut self, node_id: &[usize], state: &mut impl State) {
        match &mut self.inner[node_id[0]].kind {
            NodeKind::Single(widget, children) => {
                if node_id.len() > 1 {
                    children.update(&node_id[1..], state);
                } else {
                    widget.update(state);
                }
            }
            NodeKind::Loop(loop_node) => loop_node.body.update(&node_id[1..], state),
            _ => {}
        }
    }

    pub(crate) fn new(expressions: Rc<[Expression]>, next_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            active_loop: None,
            expr_index: 0,
            next_id,
            cache_index: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.inner
            .iter()
            .map(|node| match &node.kind {
                NodeKind::Single(_, nodes) => 1 + nodes.count(),
                NodeKind::Loop(LoopNode { body, .. }) => body.count(),
                NodeKind::ControlFlow { .. } => panic!(),
            })
            .sum()
    }

    fn reset(&mut self) {
        self.expr_index = 0;
    }

    pub fn reset_cache(&mut self) {
        self.cache_index = 0;
        for node in &mut self.inner {
            node.reset_cache();
        }
    }

    fn eval_active_loop<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope,
        layout: &mut LayoutCtx,
        f: &mut F,
    ) -> Option<Result<Size>>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>) -> Result<Size>,
    {
        if let Some(active_loop) = self.active_loop {
            let active_loop = &mut self.inner[active_loop];

            let Node {
                kind: NodeKind::Loop(loop_node),
                node_id: parent_id,
            } = active_loop
            else {
                unreachable!()
            };

            match loop_node.body.next(state, scope, layout, f) {
                result @ Some(_) => return result,
                None => {
                    if loop_node.value_index >= loop_node.collection.len() {
                        self.active_loop.take();
                        self.expr_index += 1;
                    } else {
                        loop_node.scope(scope);
                        return self.next(state, scope, layout, f);
                    }
                }
            }
        }

        None
    }

    pub fn for_each<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope<'_>,
        layout: &mut LayoutCtx,
        mut f: F,
    ) where
        F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>) -> Result<Size>,
    {
        loop {
            match self.next(state, scope, layout, &mut f) {
                Some(Ok(_)) => continue,
                _ => break,
            }
        }
    }

    fn get_cached<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope<'_>,
        layout: &mut LayoutCtx,
        f: &mut F,
    ) -> Option<Result<Size>>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>) -> Result<Size>,
    {
        let node = self.inner.get_mut(self.cache_index)?;

        match &mut node.kind {
            NodeKind::Single(widget, nodes) => {
                let data = Context::new(state, scope);
                let res = f(widget, nodes, data);
                self.cache_index += 1;
                Some(res)
            }
            NodeKind::Loop(LoopNode { body, .. }) => {
                let res = body.next(state, scope, layout, f);
                if res.is_none() {
                    self.cache_index += 1;
                }
                res
            }
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }

    pub fn next<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope<'_>,
        layout: &mut LayoutCtx,
        f: &mut F,
    ) -> Option<Result<Size>>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>) -> Result<Size>,
    {
        if let ret @ Some(_) = self.eval_active_loop(state, scope, layout, f) {
            return ret;
        }

        // Check if there is a cached value, if so: use that
        match self.get_cached(state, scope, layout, f) {
            ret @ Some(_) => return ret,
            _ => (),
        }

        let expr = self.expressions.get(self.expr_index)?;
        let mut node = match expr.eval(state, scope, self.next_id.clone()) {
            Ok(node) => node,
            Err(e) => return Some(Err(e)),
        };

        match &mut node.kind {
            NodeKind::Single(widget, nodes) => {
                self.expr_index += 1;
                let data = Context::new(state, scope);
                let res = f(widget, nodes, data);
                self.inner.push(node);
                self.cache_index = self.inner.len();
                Some(res)
            }
            NodeKind::Loop(loop_node) => {
                self.active_loop = Some(self.inner.len());
                let mut scope = scope.from_self();
                loop_node.scope(&mut scope);
                self.inner.push(node);
                self.cache_index = self.inner.len();
                self.next(state, &mut scope, layout, f)
            }
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes)> + '_ {
        self.inner
            .iter_mut()
            .map(
                |node| -> Box<dyn Iterator<Item = (&mut WidgetContainer, &mut Nodes)>> {
                    match &mut node.kind {
                        NodeKind::Single(widget, nodes) => {
                            Box::new(std::iter::once((widget, nodes)))
                        }
                        NodeKind::Loop(LoopNode { body, .. }) => Box::new(body.iter_mut()),
                        NodeKind::ControlFlow { body, .. } => Box::new(body.iter_mut()),
                    }
                },
            )
            .flatten()
    }

    pub fn first_mut(&mut self) -> Option<(&mut WidgetContainer, &mut Nodes)> {
        self.iter_mut().next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    #[test]
    fn generate_a_single_widget() {
        let mut state = ();
        let mut scope = Scope::new(None);

        let expr = expression("text", (), []);
        let mut node = expr.eval(&mut state, &mut scope, 0.into()).unwrap();
        let (widget, nodes) = node.single();

        assert_eq!(&*widget.ident, "text");
    }

    #[test]
    fn for_loop() {
        let mut state = ();
        let mut scope = Scope::new(None);

        let body = expression("text", (), []);
        let for_loop = for_expression("item", [1, 2, 3], [body]);
        let mut nodes = Nodes::new(vec![for_loop].into(), NodeId::new(0));

        let node_1 = nodes.next(&mut state, &mut scope);
        let node_2 = nodes.next(&mut state, &mut scope);
        let node_3 = nodes.next(&mut state, &mut scope);
        let node_none = nodes.next(&mut state, &mut scope);

        assert!(node_1.is_some());
        assert!(node_2.is_some());
        assert!(node_3.is_some());
        assert!(node_none.is_none());
    }
}
