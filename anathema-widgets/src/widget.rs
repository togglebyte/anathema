use std::any::Any;
use std::ops::{Deref, DerefMut};

use anathema_render::{ScreenPos, Size, Style};

use super::attributes::fields;
use super::contexts::{PaintCtx, PositionCtx, Unsized, WithSize};
use super::id::NodeId;
use super::layout::{Constraints, Padding};
use super::{AnimationCtx, Color, Display, LocalPos, Pos, Region};
use crate::contexts::{DataCtx, LayoutCtx};
use crate::error::Result;
use crate::gen::store::Store;
use crate::lookup::Lookup;
use crate::template::Template;
use crate::values::Layout;

// Layout:
// 1. Receive constraints
// 2. Layout children
// 3. Get children's suggested size
// 4. Apply offset to children
// 5. Get children's computed size
// ... paint

pub trait Widget {
    /// This should only be used for debugging, as there
    /// is nothing preventing one widget from having the same `kind` as another
    fn kind(&self) -> &'static str;

    // -----------------------------------------------------------------------------
    //     - Layout -
    // -----------------------------------------------------------------------------
    fn layout<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size>;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position<'tpl>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'tpl>]);

    fn paint<'tpl>(&mut self, ctx: PaintCtx<'_, WithSize>, children: &mut [WidgetContainer<'tpl>]);
}

pub trait AnyWidget {
    fn type_id(&self) -> std::any::TypeId;

    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn layout_any<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size>;

    fn kind_any(&self) -> &'static str;

    fn position_any<'gen: 'ctx, 'ctx>(
        &mut self,
        ctx: PositionCtx,
        children: &mut [WidgetContainer<'gen>],
    );

    fn paint_any<'gen: 'ctx, 'ctx>(
        &mut self,
        ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    );
}

impl Widget for Box<dyn AnyWidget> {
    fn kind(&self) -> &'static str {
        self.deref().kind_any()
    }

    fn layout<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        self.deref_mut().layout_any(layout)
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        self.deref_mut().position_any(ctx, children)
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        self.deref_mut().paint_any(ctx, children)
    }
}

impl<T: Widget + 'static> AnyWidget for T {
    fn type_id(&self) -> std::any::TypeId {
        Any::type_id(self)
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_any<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        self.layout(layout)
    }

    fn kind_any(&self) -> &'static str {
        self.kind()
    }

    fn position_any<'gen: 'ctx, 'ctx>(
        &mut self,
        ctx: PositionCtx,
        children: &mut [WidgetContainer<'gen>],
    ) {
        self.position(ctx, children)
    }

    fn paint_any<'gen: 'ctx, 'ctx>(
        &mut self,
        ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        self.paint(ctx, children)
    }
}

impl Widget for Box<dyn Widget> {
    fn kind(&self) -> &'static str {
        self.as_ref().kind()
    }

    fn layout<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        self.as_mut().layout(layout)
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        self.as_mut().position(ctx, children)
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        self.as_mut().paint(ctx, children)
    }
}

/// The `WidgetContainer` has to go through three steps before it can be displayed:
/// * [`layout`](Self::layout)
/// * [`position`](Self::position)
/// * [`paint`](Self::paint)
pub struct WidgetContainer<'tpl> {
    pub(crate) background: Option<Color>,
    pub(crate) display: Display,
    pub(crate) padding: Padding,
    pub(crate) animation: AnimationCtx,
    pub(crate) size: Size,
    pub(crate) templates: &'tpl [Template],
    pub(crate) children: Vec<WidgetContainer<'tpl>>,
    pub(crate) inner: Box<dyn AnyWidget>,
    pub(crate) pos: Pos,
}

impl<'tpl> WidgetContainer<'tpl> {
    pub fn new(inner: Box<dyn AnyWidget>, templates: &'tpl [Template]) -> Self {
        Self {
            templates,
            children: vec![],
            display: Display::Show,
            size: Size::ZERO,
            inner,
            pos: Pos::ZERO,
            background: None,
            padding: Padding::ZERO,
            animation: AnimationCtx::new(),
        }
    }

    pub fn new_from_widget(&self, inner: Box<dyn AnyWidget>) -> Self {
        Self {
            background: self.background,
            display: self.display,
            padding: self.padding,
            animation: self.animation.clone(),
            size: Size::ZERO,
            templates: self.templates,
            children: vec![],
            inner,
            pos: self.pos,
        }
    }

    pub fn to_ref<T: 'static>(&self) -> &T {
        let kind = self.inner.kind();

        match self.inner.deref().as_any_ref().downcast_ref::<T>() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn to_mut<T: 'static>(&mut self) -> &mut T {
        let kind = self.inner.kind();

        match self.inner.deref_mut().as_any_mut().downcast_mut::<T>() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.deref().as_any_ref().downcast_ref::<T>()
    }

    pub fn try_to_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.deref_mut().as_any_mut().downcast_mut::<T>()
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    pub fn screen_to_local(&self, screen_pos: ScreenPos) -> Option<LocalPos> {
        let pos = self.pos;

        let res = LocalPos {
            x: screen_pos.x.checked_sub(pos.x as u16)? as usize,
            y: screen_pos.y.checked_sub(pos.y as u16)? as usize,
        };

        Some(res)
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn region(&self) -> Region {
        Region::new(
            self.pos,
            Pos::new(
                self.pos.x + self.size.width as i32,
                self.pos.y + self.size.height as i32,
            ),
        )
    }

    pub fn kind(&self) -> &'static str {
        self.inner.kind()
    }

    pub fn layout<'parent>(
        &mut self,
        mut constraints: Constraints,
        values: &Store<'_>,
        lookup: &'tpl Lookup,
    ) -> Result<Size> {
        match self.display {
            Display::Exclude => self.size = Size::ZERO,
            _ => {
                let padding = self
                    .animation
                    .get_value(fields::PADDING)
                    .map(|p| Padding::new(p as usize))
                    .unwrap_or(self.padding);

                self.animation
                    .update_dst(fields::MAX_WIDTH, constraints.max_width as f32);

                constraints.max_width = self
                    .animation
                    .get_value(fields::MAX_WIDTH)
                    .map(|val| val as usize)
                    .unwrap_or(constraints.max_width);

                let layout_args = LayoutCtx::new(
                    self.templates,
                    values,
                    constraints,
                    padding,
                    &mut self.children,
                    lookup,
                );

                let size = self.inner.layout(layout_args)?;
                self.size = size;
            }
        }

        Ok(self.size)
    }

    pub fn position(&mut self, pos: Pos) {
        self.animation.update_pos(self.pos, pos);
        self.pos = self.animation.get_pos().unwrap_or(pos);
        let padding = self
            .animation
            .get_value(fields::PADDING)
            .map(|p| Padding::new(p as usize))
            .unwrap_or(self.padding);
        let ctx = PositionCtx::new(self.pos, self.size, padding);
        self.inner.position(ctx, &mut self.children);
    }

    pub fn paint(&mut self, ctx: PaintCtx<'_, Unsized>) {
        if let Display::Hide | Display::Exclude = self.display {
            return;
        }

        let mut ctx = ctx.into_sized(self.size, self.pos);
        self.paint_background(&mut ctx);
        self.inner.paint(ctx, &mut self.children);
    }

    fn paint_background(&self, ctx: &mut PaintCtx<'_, WithSize>) -> Option<()> {
        let color = self.background?;
        let width = self.size.width;

        let background_str = format!("{:width$}", "", width = width);
        let mut style = Style::new();
        style.set_bg(color);

        for y in 0..self.size.height {
            let pos = LocalPos::new(0, y);
            ctx.print(&background_str, style, pos);
        }

        Some(())
    }

    pub fn id(&self) -> NodeId {
        panic!()
    }
}