use anathema_render::Size;
use anathema_values::{Context, NodeId, Scope, State};

use super::{LoopNode, Node, NodeKind};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::Expression;
use crate::layout::{Constraints, Layout};
use crate::{Nodes, WidgetContainer};

pub struct NodeBuilder<L> {
    pub constraints: Constraints,
    layout: L,
}

impl<L: Layout> NodeBuilder<L> {
    pub fn new(constraints: Constraints, layout: L) -> Self {
        Self { constraints, layout }
    }

    pub fn layout<'e>(
        &mut self,
        node: &mut Node<'_>,
        context: &Context<'_, '_>,
    ) -> Option<Result<()>> {
        match &mut node.kind {
            NodeKind::Single(container, nodes) => self.build_single(container, nodes, context).ok()?,
            NodeKind::Loop(loop_state) => self.build_loop(loop_state, context)?,
            NodeKind::ControlFlow { .. } => self.visit_control_flow(),
        }
        Some(Ok(()))
    }

    fn build_single(
        &mut self,
        widget_container: &mut WidgetContainer,
        nodes: &mut Nodes<'_>,
        context: &Context<'_, '_>,
    ) -> Result<()> {
        let size = widget_container.layout(nodes, self.constraints, context)?;
        self.layout.layout(size);
        Ok(())
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
