use std::ops::ControlFlow;

use anathema_render::Size;
use anathema_values::{Change, Context, NodeId, Scope, State};

pub(crate) use self::controlflow::IfElse;
pub(crate) use self::loops::LoopNode;
use self::visitor::NodeVisitor;
use crate::contexts::LayoutCtx;
use crate::error::{Error, Result};
use crate::generator::expressions::Expression;
use crate::layout::Layout;
use crate::WidgetContainer;

mod controlflow;
mod loops;
pub mod visitor;

#[derive(Debug)]
pub struct Node<'e> {
    pub node_id: NodeId,
    pub(crate) kind: NodeKind<'e>,
}

impl<'e> Node<'e> {
    pub fn next<F>(
        &mut self,
        context: &Context<'_, '_>,
        layout: &LayoutCtx,
        f: &mut F,
    ) -> Result<ControlFlow<(), ()>>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, &Context<'_, '_>) -> Result<()>,
    {
        match &mut self.kind {
            NodeKind::Single(widget, children) => {
                f(widget, children, context)?;
                Ok(ControlFlow::Continue(()))
            }
            NodeKind::Loop(loop_state) => loop {
                let mut scope = context.scope.reparent();
                let Some(value) = loop_state.next_value(context) else {
                    return Ok(ControlFlow::Continue(()));
                };
                scope.scope(loop_state.binding.clone(), value);
                loop_state.body.reset();
                let context = Context::new(context.state, &scope);

                while let Some(res) = loop_state.body.next(&context, layout, f) {
                    match res? {
                        ControlFlow::Continue(()) => continue,
                        ControlFlow::Break(()) => break,
                    }
                }
            },
            NodeKind::ControlFlow(if_else) => {
                if if_else.body.is_none() {
                    if_else.load_body(context, self.node_id.child(0));
                }

                let Some(body) = if_else.body.as_mut() else {
                    return Ok(ControlFlow::Break(()));
                };

                while let Some(res) = body.next(context, layout, f) {
                    match res? {
                        ControlFlow::Continue(()) => continue,
                        ControlFlow::Break(()) => break,
                    }
                }

                Ok(ControlFlow::Continue(()))
            }
        }
    }

    fn reset_cache(&mut self) {
        match &mut self.kind {
            NodeKind::Single(_, nodes) => nodes.reset_cache(),
            NodeKind::Loop(loop_state) => loop_state.reset_cache(),
            NodeKind::ControlFlow(if_else) => if_else.reset_cache(),
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

    fn nodes(&mut self) -> &mut Nodes<'e> {
        match &mut self.kind {
            NodeKind::Single(_, nodes) => nodes,
            NodeKind::Loop(loop_state) => &mut loop_state.body,
            NodeKind::ControlFlow { .. } => panic!(),
        }
    }
}

#[cfg(test)]
impl<'e> Node<'e> {
    pub(crate) fn single(&mut self) -> (&mut WidgetContainer, &mut Nodes<'e>) {
        match &mut self.kind {
            NodeKind::Single(inner, nodes) => (inner, nodes),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeKind<'e> {
    Single(WidgetContainer, Nodes<'e>),
    Loop(LoopNode<'e>),
    ControlFlow(IfElse<'e>),
}

#[derive(Debug)]
// TODO: possibly optimise this by making nodes optional on the node
pub struct Nodes<'e> {
    expressions: &'e [Expression],
    inner: Vec<Node<'e>>,
    active_loop: Option<usize>,
    expr_index: usize,
    next_id: NodeId,
    cache_index: usize,
}

impl<'e> Nodes<'e> {
    pub fn reset(&mut self) {
        self.expr_index = 0;
    }

    pub fn advance(&mut self) {
        self.expr_index += 1;
    }

    fn new_node(&mut self, context: &Context<'_, '_>) -> Option<Result<()>> {
        let expr = self.expressions.get(self.expr_index)?;
        self.expr_index += 1;
        let Ok(node) = expr.eval(&context, self.next_id.next()) else {
            return None;
        };
        self.inner.push(node);
        Some(Ok(()))
    }

    pub fn next<F>(
        &mut self,
        context: &Context<'_, '_>,
        layout: &LayoutCtx,
        f: &mut F,
    ) -> Option<Result<ControlFlow<(), ()>>>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, &Context<'_, '_>) -> Result<()>,
    {
        match self.inner.get_mut(self.cache_index) {
            Some(n) => {
                self.cache_index += 1;
                let val = n.next(context, layout, f);
                Some(val)
            }
            None => {
                if let Err(e) = self.new_node(context)? {
                    return Some(Err(e));
                }
                self.next(context, layout, f)
            }
        }
    }

    pub fn for_each<F>(
        &mut self,
        context: &Context<'_, '_>,
        layout: &LayoutCtx,
        mut f: F,
    ) -> Result<()>
    where
        F: FnMut(&mut WidgetContainer, &mut Nodes, &Context<'_, '_>) -> Result<()>,
    {
        loop {
            if let Some(res) = self.next(context, layout, &mut f) {
                match res? {
                    ControlFlow::Continue(()) => continue,
                    ControlFlow::Break(()) => break,
                }
            }
            break;
        }
        Ok(())
    }

    // TODO: move this into a visitor?
    pub fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        update(&mut self.inner, node_id, change, state);
    }

    pub(crate) fn new(expressions: &'e [Expression], next_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            active_loop: None,
            expr_index: 0,
            next_id,
            cache_index: 0,
        }
    }

    // TODO: move this into a visitor?
    pub fn count(&self) -> usize {
        count(self.inner.iter())
    }

    // TODO: move this into a visitor?
    pub fn reset_cache(&mut self) {
        self.cache_index = 0;
        for node in &mut self.inner {
            node.reset_cache();
        }
    }

    // pub fn next_old<F>(
    //     &mut self,
    //     // state: &dyn State,
    //     // scope: &Scope<'_>,
    //     context: &Context<'_, '_>,
    //     layout: &mut LayoutCtx,
    //     f: &mut F,
    // ) -> Option<Result<Size>>
    // where
    //     F: FnMut(&mut WidgetContainer, &mut Nodes, &Context<'_, '_>) -> Result<Size>,
    // {
    //     // Evaluate the active loop if there is one
    //     if let Some(node) = self.active_loop.map(|index| &mut self.inner[index]) {
    //         match &mut node.kind {
    //             NodeKind::Loop(loop_node) => {
    //                 let mut scope = context.scope.reparent();
    //                 let binding = loop_node.binding.clone();
    //                 let value = loop_node.value(&context).unwrap();
    //                 scope.scope(binding, value);
    //                 let context = Context::new(context.state, &scope);

    //                 match loop_node.body.next_old(&context, layout, f) {
    //                     res @ Some(_) => return res,
    //                     None if loop_node.next_value() => {
    //                         return self.next_old(&context, layout, f)
    //                     }
    //                     None => {
    //                         self.active_loop.take();
    //                         return self.next_old(&context, layout, f);
    //                     }
    //                 }
    //             }
    //             _ => unreachable!("only loop nodes are stored as active loops"),
    //         }
    //     }

    //     let node = match self.inner.get_mut(self.cache_index) {
    //         Some(node) => {
    //             self.cache_index += 1;
    //             node
    //         }
    //         None => {
    //             let expr = self.expressions.get(self.expr_index)?;
    //             match expr.eval(context, self.next_id.next()) {
    //                 Ok(node) => {
    //                     self.expr_index += 1;
    //                     let index = self.inner.len();
    //                     self.inner.push(node);
    //                     self.cache_index = self.inner.len();
    //                     &mut self.inner[index]
    //                 }
    //                 Err(e) => return Some(Err(e)),
    //             }
    //         }
    //     };

    //     // TODO: next up: review this whole block.
    //     //       This is all wonky
    //     match &mut node.kind {
    //         NodeKind::Single(widget, nodes) => {
    //             // let data = Context::new(state, scope);
    //             let res = f(widget, nodes, context);
    //             Some(res)
    //         }
    //         NodeKind::Loop(_loop_node) => {
    //             // // TODO: this shouldn't be here and in the `scope` call, it's a hack
    //             // if loop_node.value_index < loop_node.collection.len() {
    //             //     // scope.push();
    //             //     if loop_node.scope(scope) {
    //             self.active_loop = Some(self.cache_index - 1);
    //             //     }
    //             // }

    //             // self.next(state, scope, layout, f)
    //             None
    //         }
    //         NodeKind::ControlFlow { .. } => panic!(),
    //     }
    // }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes<'e>)> + '_ {
        self.inner
            .iter_mut()
            .map(
                |node| -> Box<dyn Iterator<Item = (&mut WidgetContainer, &mut Nodes<'e>)>> {
                    match &mut node.kind {
                        NodeKind::Single(widget, nodes) => {
                            Box::new(std::iter::once((widget, nodes)))
                        }
                        NodeKind::Loop(loop_state) => Box::new(loop_state.iter_mut()),
                        NodeKind::ControlFlow(control_flow) => {
                            Box::new(control_flow.body.iter_mut().map(|n| n.iter_mut()).flatten())
                        }
                    }
                },
            )
            .flatten()
    }

    pub fn first_mut(&mut self) -> Option<(&mut WidgetContainer, &mut Nodes<'e>)> {
        self.iter_mut().next()
    }
}

fn count<'a>(nodes: impl Iterator<Item = &'a Node<'a>>) -> usize {
    nodes
        .map(|node| match &node.kind {
            NodeKind::Single(_, nodes) => 1 + nodes.count(),
            NodeKind::Loop(loop_state) => loop_state.count(),
            NodeKind::ControlFlow(if_else) => if_else.count(),
        })
        .sum()
}

// Apply change / update to relevant nodes
fn update(nodes: &mut [Node<'_>], node_id: &[usize], change: Change, state: &mut impl State) {
    for node in nodes {
        if node.node_id.contains(node_id) {
            if node.node_id.eq(node_id) {
                node.update(change, state);
                return;
            }

            match &mut node.kind {
                NodeKind::Single(_widget, children) => {
                    return children.update(&node_id, change, state)
                }
                NodeKind::Loop(loop_node) => return loop_node.update(node_id, change, state),
                _ => panic!("better sort this out"),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::{list, TestState};
    use anathema_values::{Value, ValueExpr};

    use super::*;
    use crate::generator::testing::*;
    use crate::layout::Constraints;
    use crate::Padding;

    #[test]
    fn generate_a_single_widget() {
        let test = expression("test", None, [], []).test();
        let mut node = test.eval().unwrap();
        let (widget, nodes) = node.single();
        assert_eq!(widget.kind(), "test");
    }

    #[test]
    fn for_loop() {
        let string = "hello".into();
        let body = expression("test", Some(string), [], []);
        let exprs = vec![for_expression("item", list([1, 2, 3]), [body])];
        let mut nodes = TestNodes::new(&exprs);
        let size = nodes.layout().unwrap();
        assert_eq!(size, Size::new(5, 3));
        assert_eq!(nodes.nodes.count(), 3);
    }

    #[test]
    fn if_else() {
        let is_true = false.into();
        let is_else = Some(false.into());

        let else_if_expr = vec![expression("test", Some("else branch".into()), [], [])];
        let if_expr = vec![expression("test", Some("true".into()), [], [])];
        let else_expr = vec![expression(
            "test",
            Some("else branch without condition".into()),
            [],
            [],
        )];

        let exprs = vec![if_expression(
            (is_true, if_expr),
            vec![(is_else, else_if_expr), (None, else_expr)],
        )];
        let mut nodes = TestNodes::new(&exprs);
        let size = nodes.layout().unwrap();
        panic!("{size:?}");
    }
}
