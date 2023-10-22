use std::any::Any;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

use anathema_render::{Color, ScreenPos, Size, Style};
use anathema_values::{remove_node, Context, NodeId, State};

use super::contexts::{PaintCtx, PositionCtx, Unsized, WithSize};
use super::layout::Constraints;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::Nodes;
use crate::{Display, LocalPos, Padding, Pos, Region};

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
    fn layout(
        &mut self,
        children: &mut Nodes,
        ctx: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size>;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx);

    fn paint(&mut self, children: &mut Nodes<'_>, mut ctx: PaintCtx<'_, WithSize>) {
        for (widget, children) in children.iter_mut() {
            let ctx = ctx.sub_context(None);
            widget.paint(children, ctx);
        }
    }

    /// Called when a value the widget subscribes to has changed.
    fn update(&mut self, _state: &mut dyn State) {}
}

pub trait AnyWidget {
    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn layout_any(
        &mut self,
        children: &mut Nodes,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size>;

    fn kind_any(&self) -> &'static str;

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx);

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>);

    fn update_any(&mut self, state: &mut dyn State);
}

impl Widget for Box<dyn AnyWidget> {
    fn kind(&self) -> &'static str {
        self.deref().kind_any()
    }

    fn layout(
        &mut self,
        children: &mut Nodes,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        self.deref_mut().layout_any(children, layout, data)
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.deref_mut().position_any(children, ctx)
    }

    fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.deref_mut().paint_any(children, ctx)
    }

    fn update(&mut self, state: &mut dyn State) {
        self.deref_mut().update_any(state)
    }
}

impl<T: Widget + 'static> AnyWidget for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_any(
        &mut self,
        children: &mut Nodes,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        self.layout(children, layout, data)
    }

    fn kind_any(&self) -> &'static str {
        self.kind()
    }

    fn position_any(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.position(children, ctx)
    }

    fn paint_any<'gen: 'ctx, 'ctx>(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.paint(children, ctx)
    }

    fn update_any(&mut self, state: &mut dyn State) {
        self.update(state)
    }
}

impl Widget for Box<dyn Widget> {
    fn kind(&self) -> &'static str {
        self.as_ref().kind()
    }

    fn layout(
        &mut self,
        children: &mut Nodes,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        self.as_mut().layout(children, layout, data)
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        self.as_mut().position(children, ctx)
    }

    fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, WithSize>) {
        self.as_mut().paint(children, ctx)
    }

    fn update(&mut self, state: &mut dyn State) {
        self.as_mut().update(state)
    }
}

/// The `WidgetContainer` has to go through three steps before it can be displayed:
/// * [`layout`](Self::layout)
/// * [`position`](Self::position)
/// * [`paint`](Self::paint)
pub struct WidgetContainer {
    pub(crate) background: Option<Color>,
    pub(crate) display: Display,
    pub(crate) padding: Padding,
    pub(crate) inner: Box<dyn AnyWidget>,
    pub(crate) pos: Pos,
    pub(crate) size: Size,
    pub(crate) node_id: NodeId,
}

impl WidgetContainer {
    pub fn kind(&self) -> &'static str {
        self.inner.kind()
    }

    pub fn to_ref<T: 'static>(&self) -> &T {
        let kind = self.inner.kind();

        match self.try_to_ref() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn to_mut<T: 'static>(&mut self) -> &mut T {
        let kind = self.inner.kind();

        match self.try_to_mut() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        let _kind = self.inner.kind();

        let any = self
            .inner
            .deref()
            .as_any_ref()
            .downcast_ref::<Box<dyn AnyWidget>>()
            .expect("this should always be a boxed AnyWidget");

        any.deref().as_any_ref().downcast_ref::<T>()
    }

    pub fn try_to_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let any = self
            .inner
            .deref_mut()
            .as_any_mut()
            .downcast_mut::<Box<dyn AnyWidget>>()
            .expect("this should always be a boxed AnyWidget");

        any.deref_mut().as_any_mut().downcast_mut::<T>()
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

    pub fn layout<'parent>(
        &mut self,
        children: &mut Nodes,
        constraints: Constraints,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        match self.display {
            Display::Exclude => self.size = Size::ZERO,
            _ => {
                let layout = LayoutCtx::new(constraints, self.padding);
                let mut size = Size::ZERO;
                self.inner.layout(children, &layout, data)?;

                // TODO: we should compare the new size with the old size
                //       to determine if the layout needs to propagate outwards
                //       or stop reflow (which ever we decide to do)

                self.size = size;
                self.size.width += self.padding.left + self.padding.right;
                self.size.height += self.padding.top + self.padding.bottom;
            }
        }

        Ok(self.size)
    }

    pub fn position(&mut self, children: &mut Nodes, pos: Pos) {
        self.pos = pos;

        let pos = Pos::new(
            self.pos.x + self.padding.left as i32,
            self.pos.y + self.padding.top as i32,
        );

        let ctx = PositionCtx::new(pos, self.inner_size());
        self.inner.position(children, ctx);
    }

    pub fn paint(&mut self, children: &mut Nodes, ctx: PaintCtx<'_, Unsized>) {
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
        self.inner.paint(children, ctx);
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

    pub fn update(&mut self, state: &mut impl State) {
        self.inner.update(state);
    }
}

impl Debug for WidgetContainer {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Drop for WidgetContainer {
    fn drop(&mut self) {
        let mut removed_node = NodeId::disposable();
        std::mem::swap(&mut self.node_id, &mut removed_node);
        remove_node(removed_node);
    }
}
