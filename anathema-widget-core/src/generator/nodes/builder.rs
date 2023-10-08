use anathema_render::Size;
use anathema_values::{Context, NodeId, Scope, State};

use super::{LoopNode, Node, NodeKind};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::Expression;
use crate::layout::Constraints;
use crate::{Nodes, WidgetContainer};

pub struct NodeBuilder {
    constraints: Constraints,
}

impl NodeBuilder {
    pub fn new(constraints: Constraints) -> Self {
        Self {
            constraints,
        }
    }

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
            NodeKind::Single(container, nodes) => {
                drop(self.build_single(container, nodes, context))
            }
            NodeKind::Loop(loop_state) => drop(self.build_loop(loop_state, context)),
            NodeKind::ControlFlow { .. } => self.visit_control_flow(),
        }

        Some(Ok(node))
    }

    fn build_single(
        &mut self,
        widget_container: &mut WidgetContainer,
        nodes: &mut Nodes<'_>,
        context: &Context<'_, '_>,
    ) -> Result<Size> {
        widget_container.layout(nodes, self.constraints, context)
    }

    fn build_loop(
        &mut self,
        loop_node: &mut LoopNode<'_>,
        context: &Context<'_, '_>,
    ) -> Option<()> {
        // Scope value.
        // If there are no more values to scope then return;

        let value = loop_node.value(context)?;
        let binding = loop_node.binding.clone();
        let mut scope = context.scope.reparent();
        scope.scope(binding, value);
        let context = Context::new(context.state, &scope);

        loop {
            let node_id = loop_node.body.next_id.next();
            let body = &mut loop_node.body;

            match body.next(self, &context) {
                Some(Ok(())) => body.advance(),
                None => {
                    body.reset();
                    self.build_loop(loop_node, &context)?;
                }
                Some(Err(err)) => panic!("ERR: {err}"),
            }
        }

        Some(())
    }

    fn visit_control_flow(&mut self) {
        todo!()
    }
}
