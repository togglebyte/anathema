use anathema_geometry::{LocalPos, Pos, Size};

use crate::layout::text::StringSession;
use crate::layout::{Constraints, LayoutCtx, PositionCtx};
use crate::paint::{PaintCtx, Unsized};
use crate::widget::{AnyWidget, PositionChildren, ValueKey};
use crate::{AttributeStorage, LayoutChildren, PaintChildren, WidgetId};

#[derive(Debug)]
pub struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub size: Size,
    pub pos: Pos,
}

impl Container {
    pub fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        self.size = self.inner.any_layout(children, constraints, self.id, ctx);
        // Floating widgets always report a zero size
        // as they should not affect their parents
        match self.inner.any_floats() {
            true => Size::ZERO,
            false => self.size,
        }
    }

    pub fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.pos = pos;
        let ctx = PositionCtx {
            inner_size: self.size,
            pos,
        };
        self.inner.any_position(children, self.id, attribute_storage, ctx);
    }

    pub fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        text: &mut StringSession<'_>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        let mut ctx = ctx.into_sized(self.size, self.pos);

        let attrs = attribute_storage.get(self.id);
        // Draw background
        if attrs.contains(&ValueKey::Attribute("background")) {
            for y in 0..self.size.height as u16 {
                for x in 0..self.size.width as u16 {
                    let pos = LocalPos::new(x, y);
                    ctx.place_glyph(' ', attrs, pos);
                }
            }
        }

        self.inner.any_paint(children, self.id, attribute_storage, ctx, text)
    }

    /// Get a mutable reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if the type is not matching the type of the widget
    pub fn to<T: 'static>(&mut self) -> &mut T {
        self.inner
            .to_any_mut()
            .downcast_mut::<T>()
            .expect("the type did not match that of the widget")
    }

    /// Get a reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if the type is not matching the type of the widget
    pub fn to_ref<T: 'static>(&self) -> &T {
        self.inner
            .to_any_ref()
            .downcast_ref::<T>()
            .expect("the type did not match that of the widget")
    }

    /// Get a mutable reference to the underlying widget of the given type
    pub fn try_to<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.to_any_mut().downcast_mut::<T>()
    }

    /// Get a reference to the underlying widget of the given type.
    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.to_any_ref().downcast_ref::<T>()
    }
}
