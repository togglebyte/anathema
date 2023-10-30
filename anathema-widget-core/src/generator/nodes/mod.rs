use std::iter::once;
use std::ops::ControlFlow;

use anathema_render::Size;
use anathema_values::{Change, Context, LocalScope, NodeId, State};

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
    pub(super) scope: LocalScope<'e>,
}

impl<'e> Node<'e> {
    pub fn next<F>(
        &mut self,
        context: &Context<'_, 'e>,
        layout: &LayoutCtx,
        f: &mut F,
    ) -> Result<ControlFlow<(), ()>>
    where
        F: FnMut(&mut WidgetContainer<'e>, &mut Nodes<'e>, &Context<'_, 'e>) -> Result<()>,
    {
        match &mut self.kind {
            NodeKind::Single(Single {
                widget, children, ..
            }) => {
                f(widget, children, context)?;
                Ok(ControlFlow::Continue(()))
            }
            NodeKind::Loop(loop_state) => loop_state.next(context, layout, f),
            NodeKind::ControlFlow(if_else) => {
                let Some(body) = if_else.body_mut() else {
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
            NodeKind::Single(Single { children, .. }) => children.reset_cache(),
            NodeKind::Loop(loop_state) => loop_state.reset_cache(),
            NodeKind::ControlFlow(if_else) => if_else.reset_cache(),
        }
    }

    // Update this node.
    // This means that the update was specifically for this node, 
    // and none of its children
    fn update(&mut self, change: Change, context: &Context<'_, '_>) {
        let scope = &self.scope;
        let context = context.reparent(scope);

        match &mut self.kind {
            NodeKind::Single(Single { widget, .. }) => widget.update(&context, &self.node_id),
            NodeKind::Loop(loop_node) => match change {
                Change::Remove(index) => loop_node.remove(index),
                Change::Add => loop_node.add(),
                _ => (),
            },
            // NOTE: the control flow it self has no immediate information
            // that needs updating, so an update should never end with the 
            // control flow node
            NodeKind::ControlFlow(_) => {}
        }
    }
}

#[cfg(test)]
impl<'e> Node<'e> {
    pub(crate) fn single(&mut self) -> (&mut WidgetContainer<'e>, &mut Nodes<'e>) {
        match &mut self.kind {
            NodeKind::Single(Single { widget, children }) => (widget, children),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Single<'e> {
    pub(crate) widget: WidgetContainer<'e>,
    pub(crate) children: Nodes<'e>,
}

#[derive(Debug)]
pub(crate) enum NodeKind<'e> {
    Single(Single<'e>),
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

    fn new_node(&mut self, context: &Context<'_, 'e>) -> Option<Result<()>> {
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
        context: &Context<'_, 'e>,
        layout: &LayoutCtx,
        f: &mut F,
    ) -> Option<Result<ControlFlow<(), ()>>>
    where
        F: FnMut(&mut WidgetContainer<'e>, &mut Nodes<'e>, &Context<'_, 'e>) -> Result<()>,
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
        context: &Context<'_, 'e>,
        layout: &LayoutCtx,
        mut f: F,
    ) -> Result<()>
    where
        F: FnMut(&mut WidgetContainer<'e>, &mut Nodes<'e>, &Context<'_, 'e>) -> Result<()>,
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
    pub fn update(&mut self, node_id: &[usize], change: Change, context: &Context<'_, '_>) {
        update(&mut self.inner, node_id, change, context);
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

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer<'e>, &mut Nodes<'e>)> + '_ {
        self.inner
            .iter_mut()
            .map(
                |node| -> Box<dyn Iterator<Item = (&mut WidgetContainer<'e>, &mut Nodes<'e>)>> {
                    match &mut node.kind {
                        NodeKind::Single(Single {
                            widget, children, ..
                        }) => Box::new(once((widget, children))),
                        NodeKind::Loop(loop_state) => Box::new(loop_state.iter_mut()),
                        NodeKind::ControlFlow(control_flow) => Box::new(control_flow.iter_mut()),
                    }
                },
            )
            .flatten()
    }

    pub fn first_mut(&mut self) -> Option<(&mut WidgetContainer<'e>, &mut Nodes<'e>)> {
        self.iter_mut().next()
    }
}

fn count<'a>(nodes: impl Iterator<Item = &'a Node<'a>>) -> usize {
    nodes
        .map(|node| match &node.kind {
            NodeKind::Single(Single { children, .. }) => 1 + children.count(),
            NodeKind::Loop(loop_state) => loop_state.count(),
            NodeKind::ControlFlow(if_else) => if_else.count(),
        })
        .sum()
}

// Apply change / update to relevant nodes
fn update(nodes: &mut [Node<'_>], node_id: &[usize], change: Change, context: &Context<'_, '_>) {
    for node in nodes {
        if node.node_id.contains(node_id) {

            // Found the node to update
            if node.node_id.eq(node_id) {
                node.update(change, context);
                return;
            }

            let scope = &node.scope;
            let context = context.reparent(scope);

            match &mut node.kind {
                NodeKind::Single(Single { children, .. }) => {
                    return children.update(&node_id, change, &context)
                }
                NodeKind::Loop(loop_node) => return loop_node.update(node_id, change, &context),
                NodeKind::ControlFlow(if_else) => return if_else.update(node_id, change, &context),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::{ident, list, TestState};
    use anathema_values::ValueExpr;

    use super::*;
    use crate::generator::testing::*;
    use crate::layout::Constraints;
    use crate::Padding;

    #[test]
    fn generate_a_single_widget() {
        let test = expression("test", None, [], []).test();
        let mut node = test.eval().unwrap();
        let (widget, nodes) = node.single();
        assert_eq!(widget.kind(), "text");
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
    fn for_loop_from_state() {
        let string = ValueExpr::Ident("item".into());
        let body = expression("test", Some(string), [], []);
        let exprs = vec![for_expression("item", ident("generic_list"), [body])];
        let mut nodes = TestNodes::new(&exprs);
        let size = nodes.layout().unwrap();
        assert_eq!(size, Size::new(1, 3));
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
