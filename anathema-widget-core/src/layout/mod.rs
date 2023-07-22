use std::fmt::{self, Display};

use anathema_render::Size;

pub use self::constraints::Constraints;

mod constraints;

// -----------------------------------------------------------------------------
//   - Re-export layouts -
// -----------------------------------------------------------------------------
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::WidgetContainer;

pub trait Layout {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        children: &mut Vec<WidgetContainer>,
        size: &mut Size,
    ) -> Result<()>;
}

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct Layouts<'ctx, 'widget, 'parent, T> {
    pub ctx: &'ctx mut LayoutCtx<'widget, 'parent>,
    pub size: Size,
    pub layout: T,
}

impl<'ctx, 'widget, 'parent, T: Layout> Layouts<'ctx, 'widget, 'parent, T> {
    pub fn new(layout: T, ctx: &'ctx mut LayoutCtx<'widget, 'parent>) -> Self {
        Self {
            ctx,
            layout,
            size: Size::ZERO,
        }
    }

    pub fn layout(&mut self, children: &mut Vec<WidgetContainer>) -> Result<&mut Self> {
        self.layout.layout(self.ctx, children, &mut self.size)?;
        Ok(self)
    }

    pub fn expand_horz(&mut self) -> &mut Self {
        self.size.width = self.ctx.constraints.max_width;
        self
    }

    pub fn expand_vert(&mut self) -> &mut Self {
        self.size.height = self.ctx.constraints.max_height;
        self
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.size)
    }
}
