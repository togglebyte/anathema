use anathema_geometry::{Pos, Region, Size};

use crate::container::Container;
use crate::layout::{Constraints, LayoutCtx, PositionFilter, Viewport};
use crate::paint::{PainFilter, PaintCtx, Unsized};
use crate::widget::{ForEach, PaintChildren, PositionChildren};
use crate::{AttributeStorage, EvalContext, LayoutForEach, LayoutChildren, WidgetId};

#[derive(Debug)]
pub struct Element<'bp> {
    pub ident: &'bp str,
    pub(crate) container: Container,
}

impl<'bp> Element<'bp> {
    pub fn id(&self) -> WidgetId {
        self.container.id
    }

    pub(crate) fn new(ident: &'bp str, container: Container) -> Self {
        Self { ident, container }
    }

    pub fn layout(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        ctx: &mut EvalContext<'_, '_, 'bp>,
    ) -> Size {
        // If the context doesn't force layout, and the id is not in the list of dirty widgets
        // (currently this path ctl) then return the cached value
        if !ctx.needs_layout(self.id()) {
            return self.size();
        }

        self.container.layout(children, constraints, ctx)
    }

    /// Position the element
    pub fn position(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        self.container.position(children, pos, attribute_storage, viewport);
    }

    /// Draw an element to the surface
    pub fn paint(
        &mut self,
        children: ForEach<'_, 'bp, PainFilter>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.container.paint(children, ctx, attribute_storage)
    }

    pub fn size(&self) -> Size {
        self.container.cache.size
    }

    /// Inner bounds in global space
    pub fn inner_bounds(&self) -> Region {
        self.container.inner_bounds
    }

    /// Get a mutable reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if the element is of a different type
    pub fn to<T: 'static>(&mut self) -> &mut T {
        self.try_to().expect("wrong element type")
    }

    /// Get a mutable reference to the underlying widget of the given type
    pub fn try_to<T: 'static>(&mut self) -> Option<&mut T> {
        self.container.inner.to_any_mut().downcast_mut::<T>()
    }

    /// Get a reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if hte element is of a different type
    pub fn to_ref<T: 'static>(&self) -> &T {
        self.try_to_ref().expect("wrong element type")
    }

    /// Get a reference to the underlying widget of the given type
    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        self.container.inner.to_any_ref().downcast_ref::<T>()
    }

    /// Get the position of the container
    pub fn get_pos(&self) -> Pos {
        self.container.pos
    }
}
