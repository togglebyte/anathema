use std::any::Any;
use std::ops::{Deref, DerefMut};

use anathema_render::{ScreenPos, Size, Style};

use super::contexts::{PaintCtx, PositionCtx, Unsized, WithSize};
use super::id::NodeId;
use super::layout::{Constraints, Padding};
use super::{Color, Display, LocalPos, Pos, Region};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::store::Store;
use crate::lookup::Lookup;
use crate::template::Template;

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
    fn kind(&self) -> &'static str {
        "[widget]"
    }

    // -----------------------------------------------------------------------------
    //     - Layout -
    // -----------------------------------------------------------------------------
    fn layout<'widget, 'tpl, 'parent>(&mut self, ctx: LayoutCtx<'widget, 'tpl, 'parent>, children: &mut Vec<WidgetContainer<'tpl>>) -> Result<Size>;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position<'tpl>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'tpl>]);

    fn paint<'tpl>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'tpl>],
    ) {
        for child in children {
            let ctx = ctx.sub_context(None);
            child.paint(ctx);
        }
    }
}

pub trait AnyWidget {
    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn layout_any<'widget, 'tpl, 'parent>(&mut self, ctx: LayoutCtx<'widget, 'tpl, 'parent>, children: &mut Vec<WidgetContainer<'tpl>>) -> Result<Size>;

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

    fn layout<'widget, 'tpl, 'parent>(&mut self, ctx: LayoutCtx<'widget, 'tpl, 'parent>, children: &mut Vec<WidgetContainer<'tpl>>) -> Result<Size> {
        self.deref_mut().layout_any(ctx, children)
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
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_any<'widget, 'tpl, 'parent>(&mut self, ctx: LayoutCtx<'widget, 'tpl, 'parent>, children: &mut Vec<WidgetContainer<'tpl>>) -> Result<Size> {
        self.layout(ctx, children)
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

    fn layout<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>, children: &mut Vec<WidgetContainer<'tpl>>) -> Result<Size> {
        self.as_mut().layout(layout, children)
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
    pub(crate) templates: &'tpl [Template],
    pub children: Vec<WidgetContainer<'tpl>>,
    pub(crate) inner: Box<dyn AnyWidget>,
    pub(crate) pos: Pos,
    size: Size,
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

    pub fn outer_size(&self) -> Size {
        self.size
    }

    pub fn inner_size(&self) -> Size {
        Size::new(
            self.size.width - (self.padding.left + self.padding.right),
            self.size.height - (self.padding.top + self.padding.bottom),
        )
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
        constraints: Constraints,
        values: &Store<'_>,
        lookup: &Lookup,
    ) -> Result<Size> {
        match self.display {
            Display::Exclude => self.size = Size::ZERO,
            _ => {
                let layout_args = LayoutCtx::new(self.templates, values, constraints, self.padding, lookup);
                let size = self.inner.layout(layout_args, &mut self.children)?;
                self.size = size;
                self.size.width += self.padding.left + self.padding.right;
                self.size.height += self.padding.top + self.padding.bottom;
            }
        }

        Ok(self.size)
    }

    pub fn position(&mut self, pos: Pos) {
        self.pos = pos;

        let pos = Pos::new(
            self.pos.x + self.padding.left as i32,
            self.pos.y + self.padding.top as i32,
        );

        let ctx = PositionCtx::new(pos, self.inner_size());
        self.inner.position(ctx, &mut self.children);
    }

    pub fn paint(&mut self, ctx: PaintCtx<'_, Unsized>) {
        if let Display::Hide | Display::Exclude = self.display {
            return;
        }

        // Paint the background without the padding,
        // using the outer size and current pos.
        let mut ctx = ctx.into_sized(self.outer_size(), self.pos);
        self.paint_background(&mut ctx);

        let pos = Pos::new(
            self.pos.x + self.padding.left as i32,
            self.pos.y + self.padding.top as i32,
        );
        ctx.update(self.inner_size(), pos);
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
