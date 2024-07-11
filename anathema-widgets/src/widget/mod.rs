use std::any::Any;
use std::fmt::{self, Debug};
use std::ops::ControlFlow;

pub type WidgetId = anathema_store::slab::Key;

use anathema_geometry::{Pos, Size};
use anathema_store::slab::SecondaryMap;
use anathema_store::tree::{Tree, TreeForEach};

pub use self::attributes::{AttributeStorage, Attributes};
pub use self::factory::Factory;
pub use self::query::{Elements, Query};
use crate::layout::text::StringSession;
use crate::layout::{Constraints, LayoutCtx, LayoutFilter, PositionCtx};
use crate::paint::{CellAttributes, PaintCtx, PaintFilter, SizePos};
use crate::WidgetKind;

mod attributes;
mod factory;
mod query;

pub struct FloatingWidgets(SecondaryMap<WidgetId, WidgetId>);

impl FloatingWidgets {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn try_remove(&mut self, key: WidgetId) {
        self.0.try_remove(key);
    }

    pub(crate) fn insert(&mut self, widget_id: WidgetId) {
        self.0.insert(widget_id, widget_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetId> {
        self.0.iter()
    }
}

pub type WidgetTree<'a> = Tree<WidgetKind<'a>>;
pub type LayoutChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, LayoutFilter<'frame, 'bp>>;
pub type PositionChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, LayoutFilter<'frame, 'bp>>;
pub type PaintChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, PaintFilter<'frame, 'bp>>;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum ValueKey<'bp> {
    #[default]
    Value,
    Attribute(&'bp str),
}

impl ValueKey<'_> {
    pub fn to_str(&self) -> &str {
        match self {
            ValueKey::Value => "value",
            ValueKey::Attribute(name) => name,
        }
    }
}

/// Any widget should never be implemented directly
/// as it's implemented for any type that implements `Widget`
pub trait AnyWidget {
    fn to_any_ref(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn any_layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size;

    fn any_position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn any_paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
        text: &mut StringSession<'_>,
    );

    fn any_floats(&self) -> bool;
}

impl<T: 'static + Widget> AnyWidget for T {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn any_layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        self.layout(children, constraints, id, ctx)
    }

    fn any_position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.position(children, id, attribute_storage, ctx)
    }

    fn any_paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
        text: &mut StringSession<'_>,
    ) {
        self.paint(children, id, attribute_storage, ctx, text)
    }

    fn any_floats(&self) -> bool {
        self.floats()
    }
}

impl Debug for dyn AnyWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dyn AnyWidget>")
    }
}

pub trait Widget {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size;

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, '_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
        // TODO make a read-only version of the buffer as it shouldn't change on paint
        text: &mut StringSession<'_>,
    ) {
        children.for_each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, text, attribute_storage);
            ControlFlow::Continue(())
        });
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn floats(&self) -> bool {
        false
    }
}

impl Debug for dyn Widget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dyn Widget>")
    }
}

pub trait WidgetRenderer {
    fn draw_glyph(&mut self, c: char, local_pos: Pos);

    fn set_attributes(&mut self, attribs: &dyn CellAttributes, local_pos: Pos);

    fn size(&self) -> Size;
}
