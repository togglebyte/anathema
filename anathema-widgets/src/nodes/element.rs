use anathema_geometry::{Pos, Rect, Size};

use crate::container::Container;
use crate::layout::{Constraints, LayoutCtx, Viewport};
use crate::paint::{PaintCtx, Unsized};
use crate::widget::{PaintChildren, PositionChildren};
use crate::{AttributeStorage, LayoutChildren, WidgetId};

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
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        self.container.layout(children, constraints, ctx)
    }

    /// Position the element
    pub fn position(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        self.container.position(children, pos, attribute_storage, viewport);
    }

    /// Draw an element to the surface
    pub fn paint(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.container.paint(children, ctx, attribute_storage)
    }

    pub fn size(&self) -> Size {
        self.container.size
    }

    pub fn inner_bounds(&self) -> Rect {
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
