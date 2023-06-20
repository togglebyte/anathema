use anathema_render::Size;

use super::many::Many;
use super::Layout;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Axis, Direction};

pub struct Horizontal(Many);

impl Horizontal {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Horizontal, 0, false);
        Self(many)
    }
}

impl Layout for Horizontal {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        self.0.layout(ctx, size)
    }
}
