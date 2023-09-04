use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::WidgetContainer;

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
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut anathema_widget_core::Nodes,
        data: anathema_values::Context<'_, '_>,
    ) -> Result<Size> {
        self.0.layout(layout, children, data)
    }

    fn finalize(&mut self, nodes: &mut anathema_widget_core::Nodes) -> Size {
        todo!()
    }
}
