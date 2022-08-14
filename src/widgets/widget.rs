use std::time::Duration;

use crate::display::{Color, ScreenPos, Size, Style};

use super::attributes::{fields, Attributes};
use super::ctx::{LayoutCtx, PaintCtx, PositionCtx, Unsized, UpdateCtx, WithSize};
use super::id::NodeId;
use super::layout::{Constraints, Padding};
use super::{AnimationCtx, Display, LocalPos, Pos, Region};

// Layout:
// 1. Receive constraints
// 2. Layout children
// 3. Get children's suggested size
// 4. Apply offset to children
// 5. Get children's computed size
// ... paint

impl Widget for Box<dyn Widget> {
    fn kind(&self) -> &'static str {
        self.as_ref().kind()
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self.as_mut().as_any()
    }

    fn needs_layout(&mut self) -> bool {
        self.as_mut().needs_layout()
    }

    fn needs_paint(&self) -> bool {
        self.as_ref().needs_paint()
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        self.as_mut().layout(ctx)
    }

    fn position(&mut self, ctx: PositionCtx) {
        self.as_mut().position(ctx)
    }

    fn paint(&mut self, ctx: PaintCtx<'_, WithSize>) {
        self.as_mut().paint(ctx)
    }

    fn flex_factor(&self) -> Option<u16> {
        self.as_ref().flex_factor()
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        self.as_mut().children()
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.as_mut().add_child(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        self.as_mut().remove_child(child_id)
    }

    fn update(&mut self, ctx: UpdateCtx) {
        self.as_mut().update(ctx);
    }
}

pub trait Widget: std::fmt::Debug + Send + Sync + 'static {
    /// This should only be used for debugging, as there
    /// is nothing preventing one widget from having the same `kind` as another
    fn kind(&self) -> &'static str;

    fn as_any(&mut self) -> &mut dyn std::any::Any;

    fn needs_layout(&mut self) -> bool {
        true
    }

    fn needs_paint(&self) -> bool {
        true
    }

    // -----------------------------------------------------------------------------
    //     - Layout -
    // -----------------------------------------------------------------------------
    fn layout(&mut self, ctx: LayoutCtx) -> Size;

    /// By the time this function is called the widget container
    /// has already set the position. This is useful to correctly set the position
    /// of the children.
    fn position(&mut self, ctx: PositionCtx);

    fn paint(&mut self, ctx: PaintCtx<'_, WithSize>);

    fn children(&mut self) -> Vec<&mut WidgetContainer>;

    fn into_container(self, id: NodeId) -> WidgetContainer
    where
        Self: Sized + 'static,
    {
        WidgetContainer::new(Box::new(self), id)
    }

    fn flex_factor(&self) -> Option<u16> {
        None
    }

    fn add_child(&mut self, widget: WidgetContainer);

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer>;

    fn update(&mut self, ctx: UpdateCtx);
}

/// The `WidgetContainer` has to go through three steps before it can be displayed:
/// * [`layout`](Self::layout)
/// * [`position`](Self::position)
/// * [`paint`](Self::paint)
///
#[derive(Debug)]
pub struct WidgetContainer {
    pub display: Display,
    pub id: NodeId,
    pub padding: Padding,
    pub(crate) size: Size,
    pub background: Option<Color>,
    pub animation: AnimationCtx,
    inner: Box<dyn Widget>,
    pos: Pos,
}

impl WidgetContainer {
    fn new(inner: Box<dyn Widget>, id: NodeId) -> Self {
        Self {
            id,
            display: Display::Show,
            size: Size::ZERO,
            inner,
            pos: Pos::ZERO,
            background: None,
            padding: Padding::ZERO,
            animation: AnimationCtx::new(),
        }
    }

    pub fn to<T: 'static>(&mut self) -> &mut T {
        let kind = self.inner.kind();
        match self.inner.as_any().downcast_mut::<T>() {
            Some(t) => t,
            None => panic!("invalid widget type, found `{kind}`"),
        }
    }

    pub fn try_to<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.as_any().downcast_mut::<T>()
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
        Region::new(self.pos, Pos::new(self.pos.x + self.size.width as i32, self.pos.y + self.size.height as i32))
    }

    pub fn kind(&self) -> &'static str {
        self.inner.kind()
    }

    pub fn animate(&mut self, delta: Duration) {
        self.animation.update(delta);

        for child in self.inner.children() {
            child.animate(delta);
        }

        let attributes = self.animation.attributes();
        if !attributes.is_empty() {
            if let Some(left) = attributes.padding_left() {
                self.padding.left = left;
            }
            if let Some(right) = attributes.padding_right() {
                self.padding.right = right;
            }
            if let Some(top) = attributes.padding_top() {
                self.padding.top = top;
            }
            if let Some(bottom) = attributes.padding_bottom() {
                self.padding.bottom = bottom;
            }
        }

        let ctx = UpdateCtx::new(attributes, self.pos, self.size);
        self.inner.update(ctx);
    }

    pub fn layout(&mut self, mut constraints: Constraints, force_layout: bool) -> Size {
        if self.inner.needs_layout() || force_layout {
            match self.display {
                Display::Exclude => self.size = Size::ZERO,
                _ => {
                    let padding = self
                        .animation
                        .get_value(fields::PADDING)
                        .map(|p| Padding::new(p as usize))
                        .unwrap_or(self.padding);

                    self.animation.update_dst(fields::MAX_WIDTH, constraints.max_width as f32);
                    constraints.max_width = self
                        .animation
                        .get_value(fields::MAX_WIDTH)
                        .map(|val| val as usize)
                        .unwrap_or(constraints.max_width);
                    let ctx = LayoutCtx::new(constraints, force_layout, padding);
                    let size = self.inner.layout(ctx);
                    self.size = size;
                }
            }
        }

        self.size
    }

    pub fn position(&mut self, pos: Pos) {
        self.animation.update_pos(self.pos, pos);
        self.pos = self.animation.get_pos().unwrap_or(pos);
        let padding =
            self.animation.get_value(fields::PADDING).map(|p| Padding::new(p as usize)).unwrap_or(self.padding);
        let ctx = PositionCtx::new(self.pos, self.size, padding);
        self.inner.position(ctx);
    }

    pub fn paint(&mut self, ctx: PaintCtx<'_, Unsized>) {
        if let Display::Hide | Display::Exclude = self.display {
            return;
        }

        let mut ctx = ctx.into_sized(self.size, self.pos);
        self.paint_background(&mut ctx);
        self.inner.paint(ctx);
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
        self.id.clone()
    }

    pub fn by_id<T: PartialEq<NodeId> + ?Sized>(&mut self, id: &T) -> Option<&mut WidgetContainer> {
        if id == &self.id {
            Some(self)
        } else {
            for child in self.inner.children() {
                if let Some(c) = child.by_id(id) {
                    return Some(c);
                }
            }
            None
        }
    }

    pub fn find_parent(&mut self, id: &NodeId) -> Option<&mut WidgetContainer> {
        if self.id.eq(id) {
            return None;
        }

        for child in self.inner.children() {
            if let Some(_c) = child.by_id(id) {
                return Some(self);
            }
        }
        None
    }

    pub fn by_type<T: 'static>(&mut self) -> Option<&mut WidgetContainer> {
        if self.try_to::<T>().is_some() {
            return Some(self);
        }

        for child in self.inner.children() {
            if let Some(c) = child.by_type::<T>() {
                return Some(c);
            }
        }
        None
    }

    pub fn at_coords<F>(&mut self, pos: ScreenPos, mut f: F) -> bool
    where
        F: FnMut(&mut Self) -> bool,
    {
        self.widget_at_coords(pos, &mut f)
    }

    fn widget_at_coords<F>(&mut self, pos: ScreenPos, f: &mut F) -> bool
    where
        F: FnMut(&mut Self) -> bool,
    {
        if self.region().contains(Pos::new(pos.x as i32, pos.y as i32)) {
            for child in self.inner.children() {
                if !child.widget_at_coords(pos, f) {
                    return false;
                }
            }
            f(self)
        } else {
            true
        }
    }

    pub fn stringify(&mut self) -> String {
        to_string(self, 0)
    }

    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;
    }

    pub fn add_child(&mut self, widget: Self) {
        self.inner.add_child(widget);
    }

    pub fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        self.inner.remove_child(child_id)
    }

    pub fn update(&mut self, mut attributes: Attributes) {
        if attributes.is_empty() {
            return;
        }

        if attributes.has(fields::BACKGROUND) {
            self.background = attributes.background();
        }

        if attributes.has(fields::DISPLAY) {
            self.display = attributes.display();
        }

        attributes.inner.retain(|k, v| {
            let value = match v.to_signed_int() {
                Some(val) => val as f32,
                None => return true,
            };

            !self.animation.update_dst(k, value)
        });

        // Padding
        if let Some(left) = attributes.padding_left() {
            self.padding.left = left;
        }
        if let Some(right) = attributes.padding_right() {
            self.padding.right = right;
        }
        if let Some(top) = attributes.padding_top() {
            self.padding.top = top;
        }
        if let Some(bottom) = attributes.padding_bottom() {
            self.padding.bottom = bottom;
        }

        let ctx = UpdateCtx::new(attributes, self.pos, self.size);
        self.inner.update(ctx);
    }
}

fn to_string(node: &mut WidgetContainer, level: usize) -> String {
    let padding = " ".repeat(level * 4);
    let mut string = format!("{padding}{} {}\n", node.kind(), node.id);

    for child in node.inner.children() {
        let s = to_string(child, level + 1);
        string.push_str(&s);
    }

    string
}
