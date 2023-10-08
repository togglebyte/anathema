use anathema_values::{Context, NodeId, Scope, State};

use super::{LoopNode, Node, NodeKind};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::Expression;
use crate::{Nodes, WidgetContainer};

pub trait NodeVisitor {
    type Output;

    fn visit(&mut self, node: &mut Node<'_>) -> Self::Output;
    fn visit_single(
        &mut self,
        widget_container: &mut WidgetContainer,
        nodes: &mut Nodes<'_>,
    ) -> Self::Output;
    fn visit_loop(&mut self, loop_node: &mut LoopNode<'_>) -> Self::Output;
    fn visit_control_flow(&mut self) -> Self::Output;
}

pub struct NodeBuilder<'a, 'val> {
    pub layout: LayoutCtx,
    pub context: Context<'a, 'val>,
}

impl<'a, 'val> NodeBuilder<'a, 'val> {
    pub fn build<'e>(
        &mut self,
        expr: &'e Expression,
        context: &Context<'_, '_>,
        node_id: NodeId,
    ) -> Option<Result<Node<'e>>> {

        let mut node = match expr.eval(&context, node_id) {
            Ok(node) => node,
            Err(e) => return Some(Err(e)),
        };

        match &mut node.kind {
            NodeKind::Single(container, nodes) => self.build_single(container, nodes, context),
            NodeKind::Loop(loop_state) => self.build_loop(loop_state, context),
            NodeKind::ControlFlow { .. } => self.visit_control_flow(),
        }

        Some(Ok(node))
    }

    fn build_single(
        &mut self,
        widget_container: &mut WidgetContainer,
        nodes: &mut Nodes<'_>,
        context: &Context<'_, '_>,
    ) -> () {
        // Perform layout
        todo!()
    }

    fn build_loop(&mut self, loop_node: &mut LoopNode<'_>, context: &Context<'_, '_>) -> () {
        // Scope value
        let value = loop_node.value(&self.context).unwrap();
        let binding = loop_node.binding.clone();
        let mut scope = context.scope.reparent();
        scope.scope(binding, value);
        let context = Context::new(context.state, &scope);

        loop {
            let node_id = loop_node.body.next_id.next();
            let body = &mut loop_node.body;

            match body.next(self, &context) {
                Some(Ok(())) => body.advance(),
                None => body.reset(),
                Some(Err(err)) => panic!("{err}"),
            }
        }
    }

    fn visit_control_flow(&mut self) {
        todo!()
    }
}
