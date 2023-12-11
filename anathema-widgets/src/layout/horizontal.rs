use anathema_render::Size;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::LayoutNodes;

use super::many::Many;

pub struct Horizontal(Many);

impl Horizontal {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Horizontal, 0, false);
        Self(many)
    }
}

impl Layout for Horizontal {
    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        self.0.layout(nodes)
    }
}
