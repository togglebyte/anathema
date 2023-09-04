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

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub(crate) kind: NodeKind,
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
    active_loop: Option<Box<Node>>,
    expr_index: usize,
    next_id: NodeId,
    node_index: usize,
}

impl Nodes {
    pub(crate) fn new(expressions: Rc<[Expression]>, next_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            active_loop: None,
            expr_index: 0,
            next_id,
            node_index: 0,
        }
    }

    fn reset(&mut self) {
        self.expr_index = 0;
        self.node_index = 0;
    }

    fn eval_active_loop<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope,
        layout: &mut LayoutCtx,
        f: &mut F,
    ) -> Option<Result<bool>>
        where F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>)
    {
        if let Some(active_loop) = self.active_loop.as_mut() {
            let Node {
                kind: NodeKind::Loop(loop_node),
                node_id: parent_id,
            } = active_loop.deref_mut()
            else {
                unreachable!()
            };

            match loop_node.body.next(state, scope, layout, f) {
                result @ Some(_) => return result,
                None => {
                    if loop_node.value_index + 1 == loop_node.collection.len() {
                        self.inner.push(*self.active_loop.take().expect(""));
                        self.expr_index += 1;
                    } else {
                        let mut scope = scope.from_self();
                        scope.scope_collection(
                            loop_node.binding.clone(),
                            &loop_node.collection,
                            loop_node.value_index,
                        );
                        loop_node.body.reset();
                        loop_node.value_index += 1;

                        return self.next(state, &mut scope, layout, f);
                    }
                }
            }
        }

        None
    }

    pub fn for_each<F>(&mut self, state: &mut dyn State, scope: &mut Scope<'_>, mut layout: LayoutCtx, mut f: F) 
        where F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>)
    {
        loop {
            let cont = self.next(state, scope, &mut layout, &mut f).unwrap().unwrap();
            if !cont {
                break;
            }
        }
    }

    pub fn next<F>(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope<'_>,
        layout: &mut LayoutCtx,
        f: &mut F
    ) -> Option<Result<bool>> 
        where F: FnMut(&mut WidgetContainer, &mut Nodes, Context<'_, '_>)
    {
        if let ret @ Some(_) = self.eval_active_loop(state, scope, layout, f) {
            return Some(Ok(true));
            // return ret;
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
                f(widget, nodes, data);
                panic!();
                // let size = widget.layout(nodes, layout.constraints, data);
                // Some(size)
            }
            NodeKind::Loop { .. } => {
                self.active_loop = Some(node.into());
                self.next(state, scope, layout, f)
            }
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }

    pub fn next_old_thing(
        &mut self,
        state: &mut dyn State,
        scope: &mut Scope<'_>,
        layout: &mut LayoutCtx,
    ) -> Option<Result<Size>> {
        panic!()
        // if let ret @ Some(_) = self.eval_active_loop(state, scope, layout) {
        //     return ret;
        // }

        // let expr = self.expressions.get(self.expr_index)?;
        // let mut node = match expr.eval(state, scope, self.next_id.clone()) {
        //     Ok(node) => node,
        //     Err(e) => return Some(Err(e)),
        // };

        // match &mut node.kind {
        //     NodeKind::Single(widget, nodes) => {
        //         self.expr_index += 1;
        //         let data = Context::new(state, scope);
        //         let size = widget.layout(nodes, layout.constraints, data);
        //         Some(size)
        //     }
        //     NodeKind::Loop { .. } => {
        //         self.active_loop = Some(node.into());
        //         self.next(state, scope, layout)
        //     }
        //     NodeKind::ControlFlow { .. } => panic!(),
        // }
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
