use std::collections::VecDeque;

use display::Size;

use crate::attributes::{fields, Attributes};
use crate::layout::Constraints;
use crate::{NodeId, Pos, Region};
use super::{Axis, LayoutCtx, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// A `ScrollView` can contain more widgets than what is visible.
#[derive(Debug)]
pub struct ScrollView {
    /// Maximum number of children.
    /// Once this number is reached, the child at the bottom of the list will be removed
    pub max_children: Option<usize>,
    /// Layout axis.
    pub axis: Axis,
    /// Should the widget auto scroll once new children are added
    pub auto_scroll: bool,
    /// Scroll offset
    pub offset: i32,
    /// Reverse the output
    pub reverse: bool,
    /// Children
    pub children: VecDeque<WidgetContainer>,

    at_the_end: bool,
    visible_children: Vec<NodeId>,
    should_auto_scroll: bool,
}

impl ScrollView {
    /// Create a new instance of a `ScrollView`
    pub fn new(
        max_children: Option<usize>,
        axis: Axis,
        offset: i32,
        auto_scroll: bool,
        reverse: bool,
    ) -> Self {
        Self {
            axis,
            max_children,
            auto_scroll,
            offset,
            reverse,

            at_the_end: true,
            children: VecDeque::new(),
            visible_children: vec![],
            should_auto_scroll: false,
        }
    }

    /// Scroll backwards
    pub fn scroll_back(&mut self, offset: i32) {
        self.offset -= offset;
    }

    /// Scroll forward
    pub fn scroll_forward(&mut self, offset: i32) {
        self.offset += offset;
    }

    fn clipping_region(&self, global_pos: Pos, size: Size) -> Region {
        Region::new(
            global_pos,
            Pos::new(
                global_pos.x + size.width as i32,
                global_pos.y + size.height as i32,
            ),
        )
    }
}

impl Widget for ScrollView {
    fn kind(&self) -> &'static str {
        "Viewport"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        let axis = self.axis;
        let mut layout = Layout::new(axis);
        layout.layout(ctx, self.children.iter_mut())
    }

    fn position(&mut self, ctx: PositionCtx) {
        // self.should_auto_scroll = self.auto_scroll && !self.reverse;

        // If the scroll position is below zero, set it to zero to prevent
        // scrolling past the first child
        if self.offset < 0 {
            self.offset = 0;
        }

        let total_child_size = match self.axis {
            Axis::Vertical => self.children.iter().map(|c| c.size.height).sum::<usize>() as i32,
            Axis::Horizontal => self.children.iter().map(|c| c.size.width).sum::<usize>() as i32,
        };

        let axis_size = match self.axis {
            Axis::Vertical => ctx.size.height as i32,
            Axis::Horizontal => ctx.size.width as i32,
        };

        // If the total child size is larger than what fits
        // in the viewport, only then is the offset relevant
        if total_child_size > axis_size {
            if self.offset + axis_size >= total_child_size || self.should_auto_scroll {
                self.offset = total_child_size - axis_size;
                self.should_auto_scroll = false;
                self.at_the_end = true;
            } else {
                self.at_the_end = false;
            }
        }

        let offset = if total_child_size > axis_size {
            self.offset
        } else {
            0
        };
        let mut positioning = Position::new(self.axis, offset);

        let visible = match self.reverse {
            false => positioning.position(ctx.pos, ctx.size, self.children.iter_mut()),
            true => positioning.position(ctx.pos, ctx.size, self.children.iter_mut().rev()),
        };

        self.visible_children = visible;
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let clip = self.clipping_region(ctx.global_pos, ctx.local_size);

        for child in self.children.iter_mut() {
            if !self.visible_children.contains(&child.id()) {
                continue;
            }
            let ctx = ctx.sub_context(Some(&clip));
            child.paint(ctx);
        }
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        self.children.iter_mut().collect()
    }

    fn add_child(&mut self, child: WidgetContainer) {
        // Ensure that we don't exceed max children
        if let Some(max_children) = self.max_children {
            while self.children.len() >= max_children && max_children > 0 {
                self.children.pop_front();
            }
        }

        self.children.push_back(child);
        if self.at_the_end && self.auto_scroll {
            self.should_auto_scroll = true;
        }
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(pos) = self.children.iter().position(|c| c.id.eq(child_id)) {
            return self.children.remove(pos);
        }
        None
    }

    fn update(&mut self, attributes: Attributes) {
        for (k, _) in &attributes {
            match k.as_str() {
                fields::MAX_CHILDREN => self.max_children = attributes.max_children(),
                fields::AXIS => self.axis = attributes.axis().unwrap_or(Axis::Vertical),
                fields::AUTO_SCROLL => self.auto_scroll = attributes.auto_scroll(),
                fields::OFFSET => self.offset = attributes.offset(),
                fields::REVERSE => self.reverse = attributes.reverse(),
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
struct Layout {
    axis: Axis,
}

impl Layout {
    fn new(axis: Axis) -> Self {
        Self { axis }
    }

    fn layout<'a>(
        &'a mut self,
        ctx: LayoutCtx,
        children: impl Iterator<Item = &'a mut WidgetContainer>,
    ) -> Size {
        // -----------------------------------------------------------------------------
        //     - Layout children -
        // -----------------------------------------------------------------------------
        let (mut child_width, mut child_height) = match self.axis {
            Axis::Horizontal => (ctx.constraints.max_width, 0),
            Axis::Vertical => (0, ctx.constraints.max_height),
        };

        let child_constraints = match self.axis {
            Axis::Horizontal => Constraints::new(None, ctx.constraints.max_height),
            Axis::Vertical => Constraints::new(ctx.constraints.max_width, None),
        };

        for child in children {
            let size = child.layout(child_constraints, ctx.force_layout);

            match self.axis {
                Axis::Horizontal => child_height = child_height.max(size.height),
                Axis::Vertical => child_width = child_width.max(size.width),
            }
        }

        Size::new(child_width, child_height)
    }
}

struct Position {
    axis: Axis,
    offset: i32,
}

impl Position {
    fn new(axis: Axis, offset: i32) -> Self {
        Self { axis, offset }
    }

    fn offset_pos(&self) -> Pos {
        match self.axis {
            Axis::Horizontal => Pos::new(self.offset, 0),
            Axis::Vertical => Pos::new(0, self.offset),
        }
    }

    fn position<'a>(
        &'a mut self,
        pos: Pos,
        size: Size,
        children: impl Iterator<Item = &'a mut WidgetContainer>,
    ) -> Vec<NodeId> {
        let viewport_region = Region::new(
            pos,
            Pos::new(pos.x + size.width as i32, pos.y + size.height as i32),
        );

        let mut next_pos = pos - self.offset_pos();

        let mut visible = vec![];

        for child in children {
            child.position(next_pos);
            match self.axis {
                Axis::Horizontal => next_pos.x += child.size.width as i32,
                Axis::Vertical => next_pos.y += child.size.height as i32,
            }

            if viewport_region.intersects(&child.region()) {
                visible.push(child.id());
            }
        }

        visible
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::test_widget;
    use crate::{Border, BorderStyle, Sides, Text};

    fn test_viewport(viewport: impl Widget, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        border.child = Some(viewport.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn vertical_scroll() {
        let mut viewport = ScrollView::new(None, Axis::Vertical, 0, false, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───────┐
            │0      │
            │1      │
            │2      │
            └───────┘
            "#,
        );
    }

    #[test]
    fn horizontal_scroll() {
        let mut viewport = ScrollView::new(None, Axis::Horizontal, 0, false, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───┐
            │012│
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn vertical_scroll_down() {
        let mut viewport = ScrollView::new(None, Axis::Vertical, 2, false, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───────┐
            │2      │
            │3      │
            │4      │
            └───────┘
            "#,
        );
    }

    #[test]
    fn horizontal_scroll_right() {
        let mut viewport = ScrollView::new(None, Axis::Horizontal, 2, false, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───┐
            │234│
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn vertical_auto_scroll_down() {
        let mut viewport = ScrollView::new(None, Axis::Vertical, 0, true, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───────┐
            │2      │
            │3      │
            │4      │
            └───────┘
            "#,
        );
    }

    #[test]
    fn horizontal_auto_scroll_down() {
        let mut viewport = ScrollView::new(None, Axis::Horizontal, 0, true, false);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───┐
            │234│
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn vertical_reverse_auto_scroll() {
        let mut viewport = ScrollView::new(None, Axis::Vertical, 0, false, true);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───────┐
            │4      │
            │3      │
            │2      │
            └───────┘
            "#,
        );
    }

    #[test]
    fn horizontal_reverse_auto_scroll() {
        let mut viewport = ScrollView::new(None, Axis::Horizontal, 0, false, true);
        for i in 0..5 {
            viewport.add_child(Text::with_text(i.to_string()).into_container(NodeId::auto()));
        }
        test_viewport(
            viewport,
            r#"
            ┌───┐
            │432│
            │   │
            │   │
            └───┘
            "#,
        );
    }
}
