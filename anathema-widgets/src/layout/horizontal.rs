use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::WidgetContainer;

use super::many::Many;

pub struct Horizontal(Many);

impl Horizontal {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Horizontal, 0, false);
        Self(many)
    }
}

impl Layout for Horizontal {
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut anathema_widget_core::Nodes,
        data: anathema_values::Context<'_, '_>,
    ) -> Result<Size> {
        self.0.layout(layout, children, data)
    }
}
