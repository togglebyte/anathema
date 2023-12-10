use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::{LayoutNodes, Nodes, WidgetContainer};

use super::many::Many;

#[derive(Debug)]
pub struct Vertical(Many);

impl Vertical {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Vertical, 0, false);
        Self(many)
    }
}

impl Layout for Vertical {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        self.0.layout(nodes)
    }
}
