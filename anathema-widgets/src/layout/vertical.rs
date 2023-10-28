use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::{WidgetContainer, Nodes};

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
    fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        layout: &LayoutCtx,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        self.0.layout(children, layout, data)
    }
}
