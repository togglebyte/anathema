use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

use anathema_render::{Color, ScreenPos, Size, Style};
use anathema_values::{remove_node, Attributes, Context, NodeId, Value};

use super::{AnyWidget, Widget};
use crate::contexts::{LayoutCtx, PaintCtx, PositionCtx, Unsized, WithSize};
use crate::error::Result;
use crate::expressions::Expression;
use crate::layout::Constraints;
use crate::nodes::Nodes;
use crate::{Display, LayoutNodes, LocalPos, Padding, Pos, Region};

/// The `WidgetContainer` has to go through three steps before it can be displayed:
/// * [`layout`](Self::layout)
/// * [`position`](Self::position)
/// * [`paint`](Self::paint)
#[derive(Debug)]
pub struct WidgetContainer<'e> {
    pub(crate) background: Value<Color>,
    pub(crate) display: Value<Display>,
    pub(crate) padding: Padding,
    pub(crate) inner: Box<dyn AnyWidget>,
    pub(crate) pos: Pos,
    pub(crate) size: Size,
    // TODO: do we need the node_id here?
    pub(crate) node_id: NodeId,
    pub expr: Option<&'e Expression>,
    pub attributes: &'e Attributes,
}

impl WidgetContainer<'_> {
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

    pub fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        constraints: Constraints,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        match self.display.value_or_default() {
            Display::Exclude => self.size = Size::ZERO,
            _ => {
                let layout = LayoutCtx::new(constraints, self.padding);
                let mut nodes = LayoutNodes::new(children, constraints, self.padding, data);
                let size = self.inner.layout(&mut nodes)?;

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
        if let Display::Hide | Display::Exclude = self.display.value_or_default() {
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
        let color = self.background.value_ref()?;
        let width = self.size.width;

        let background_str = format!("{:width$}", "", width = width);
        let mut style = Style::new();
        style.set_bg(*color);

        for y in 0..self.size.height {
            let pos = LocalPos::new(0, y);
            ctx.print(&background_str, style, pos);
        }

        Some(())
    }

    pub fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.display.resolve(context, Some(node_id));
        self.inner.update(context, node_id);
    }
}

impl Drop for WidgetContainer<'_> {
    fn drop(&mut self) {
        let mut removed_node = NodeId::disposable();
        std::mem::swap(&mut self.node_id, &mut removed_node);
        remove_node(removed_node);
    }
}
