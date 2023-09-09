use std::borrow::Cow;
use std::ops::DerefMut;
use std::rc::Rc;

use anathema_render::Size;
use anathema_values::{Change, Collection, Context, NodeId, Path, Scope, ScopeValue, State};

use self::controlflow::{Else, If};
pub(crate) use self::loops::LoopNode;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::expressions::Expression;
use crate::WidgetContainer;

mod controlflow;
mod loops;

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub(crate) kind: NodeKind,
}

impl Node {
    fn reset_cache(&mut self) {
        match &mut self.kind {
            NodeKind::Single(_, nodes) => nodes.reset_cache(),
            NodeKind::Loop(loop_state) => loop_state.reset_cache(),
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }

    fn update(&mut self, change: Change, state: &mut impl State) {
        match &mut self.kind {
            NodeKind::Single(widget, _) => widget.update(state),
            NodeKind::Loop(loop_node) => match change {
                Change::Remove(index) => loop_node.remove(index),
                Change::Add => loop_node.add(),
                _ => (),
            },
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
    inner: Vec<Node>,
    active_loop: Option<usize>,
    expr_index: usize,
    next_id: NodeId,
    cache_index: usize,
}

impl Nodes {
    fn next_expr(&mut self) {
        if self.expr_index == self.expressions.len() {
            self.expr_index = 0;
        } else {
            self.expr_index += 1;
        }
    }

    pub fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        for node in &mut self.inner {
            if node.node_id.contains(node_id) {
                if node.node_id.eq(node_id) {
                    node.update(change, state);
                    return;
                }

                match &mut node.kind {
                    NodeKind::Single(widget, children) => {
                        return children.update(&node_id, change, state)
                    }
                    NodeKind::Loop(loop_node) => return loop_node.update(node_id, change, state),
                    _ => panic!("better sort this out"),
                }
            }
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
                NodeKind::Loop(loop_state) => loop_state.count(),
                NodeKind::ControlFlow { .. } => panic!(),
            })
            .sum()
    }

    pub fn reset_cache(&mut self) {
        self.cache_index = 0;
        for node in &mut self.inner {
            node.reset_cache();
        }
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
        // Evaluate the active loop if there is one
        if let Some(node) = self.active_loop.map(|index| &mut self.inner[index]) {
            match &mut node.kind {
                NodeKind::Loop(loop_node) => match loop_node.body.next(state, scope, layout, f) {
                    res @ Some(_) => return res,
                    None if loop_node.scope(scope) => return self.next(state, scope, layout, f),
                    None => {
                        self.active_loop.take();
                        return self.next(state, scope, layout, f);
                    }
                },
                _ => unreachable!("only loop nodes are stored as active loops"),
            }
        }

        let mut node = match self.inner.get_mut(self.cache_index) {
            Some(node) => {
                self.cache_index += 1;
                node
            }
            None => {
                let expr = self.expressions.get(self.expr_index)?;
                match expr.eval(state, scope, self.next_id.next()) {
                    Ok(node) => {
                        self.expr_index += 1;
                        let index = self.inner.len();
                        self.inner.push(node);
                        self.cache_index = self.inner.len();
                        &mut self.inner[index]
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
        };

        match &mut node.kind {
            NodeKind::Single(widget, nodes) => {
                let data = Context::new(state, scope);
                let res = f(widget, nodes, data);
                Some(res)
            }
            NodeKind::Loop(loop_node) => {
                if loop_node.value_index < loop_node.collection.len() {
                    let mut scope = scope.from_self();
                    if loop_node.scope(&mut scope) {
                        self.active_loop = Some(self.cache_index - 1);
                    }
                    return self.next(state, &mut scope, layout, f);
                }

                self.next(state, scope, layout, f)
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
                        NodeKind::Loop(loop_state) => Box::new(loop_state.iter_mut()),
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
    use crate::generator::testing::*;
    use crate::layout::Constraints;
    use crate::Padding;

    #[test]
    fn generate_a_single_widget() {
        register_test_widget();
        let mut state = ();
        let mut scope = Scope::new(None);

        let expr = expression("test", None, [], []);
        let mut node = expr.eval(&mut state, &mut scope, 0.into()).unwrap();
        let (widget, nodes) = node.single();

        assert_eq!(widget.kind(), "test");
    }

    #[test]
    fn for_loop() {
        register_test_widget();
        let mut state = ();
        let mut scope = Scope::new(None);
        let mut layout = LayoutCtx::new(Constraints::unbounded(), Padding::ZERO);

        let body = expression("test", None, [], []);
        let for_loop = for_expression("item", [1, 2, 3], [body]);
        let mut nodes = Nodes::new(vec![for_loop].into(), NodeId::new(0));

        nodes.for_each(&mut state, &mut scope, &mut layout, |_, _, _| { Ok(Size::ZERO) });
        panic!("this isn't done!");

        // let node_1 = nodes.next(&mut state, &mut scope, &mut layout, &mut |_, _, _| { Ok(Size::ZERO) });
        // let node_2 = nodes.next(&mut state, &mut scope, &mut layout, &mut |_, _, _| { Ok(Size::ZERO) });
        // let node_3 = nodes.next(&mut state, &mut scope, &mut layout, &mut |_, _, _| { Ok(Size::ZERO) });
        // let node_none = nodes.next(&mut state, &mut scope, &mut layout, &mut |_, _, _| { Ok(Size::ZERO) });

        // assert!(node_1.is_some());
        // assert!(node_2.is_some());
        // assert!(node_3.is_some());
        // assert!(node_none.is_none());
    }
}
