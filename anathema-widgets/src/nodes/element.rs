use anathema_geometry::{Pos, Size};

use crate::container::Container;
use crate::layout::{Constraints, Display, LayoutCtx, TextBuffer};
use crate::paint::{PaintCtx, Unsized};
use crate::widget::{PaintChildren, PositionChildren};
use crate::{AttributeStorage, LayoutChildren, WidgetId};

pub struct Element<'bp> {
    pub ident: &'bp str,
    pub container: Container,
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

    pub fn paint(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        text_buffer: &mut TextBuffer,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.container.paint(children, ctx, text_buffer, attribute_storage)
    }

    pub fn position(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.container.position(children, pos, attribute_storage);
    }

    pub fn size(&self) -> Size {
        self.container.size
    }

    pub(crate) fn display(&self) -> Display {
        self.container.display
    }

    pub fn to_inner_ref<T: 'static>(&self) -> Option<(&T, WidgetId)> {
        let widget = self.container.inner.to_any_ref().downcast_ref()?;
        let id = self.container.id;
        Some((widget, id))
    }

    pub fn to_inner_mut<T: 'static>(&mut self) -> Option<(&mut T, WidgetId)> {
        let widget = self.container.inner.to_any_mut().downcast_mut()?;
        let id = self.container.id;
        Some((widget, id))
    }
}
