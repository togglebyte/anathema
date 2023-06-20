use anathema_render::Size;

use super::many::Many;
use super::Layout;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Axis, Direction};

#[derive(Debug)]
pub struct Vertical(Many);

impl Vertical {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Vertical, 0, false);
        Self(many)
    }
}

impl Layout for Vertical {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        self.0.layout(ctx, size)
    }
}
