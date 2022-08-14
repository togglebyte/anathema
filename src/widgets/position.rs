use crate::display::Size;

use super::{
    HorzEdge, LayoutCtx, NodeId, PaintCtx, Pos, PositionCtx, UpdateCtx, VertEdge, Widget, WidgetContainer, WithSize,
};
use crate::widgets::attributes::fields;

/// If the horizontal edge is set to `Right` the widget will expand to fill all available space
/// on the horizontal axis.
/// Same is true if the `VertEdge::Bottom` is set.
///
/// Position on the horizontal axis:
/// Left 0 would mean the left edge of the widget is positioned at the `left` value.
/// Right 0 would mean the right edge of the widget is positioned at the `right` value, where zero
/// is closest to the right.
/// ```text
/// ----- total width -----
/// ┌─┐                 ┌─┐
/// │1│                 │2│
/// └─┘                 └─┘
///
/// Position on the vertical axis:
/// Top 0 would mean the top edge of the child is positioned at `top` value.
/// Bottom 0 would mean the bottom edge of the child is positioned at the `bottom` value, where
/// zero is closest to the bottom.
/// ```text
/// | ┌────────┐
/// | │1       │
/// | └────────┘
/// |
/// |
/// |
/// |
/// |
/// | ┌────────┐
/// | │2       │
/// | └────────┘
/// ```
/// ```
#[derive(Debug)]
pub struct Position {
    /// Child widget
    pub child: Option<WidgetContainer>,
    /// Horizontal edge
    pub horz_edge: HorzEdge,
    /// Vertical edge
    pub vert_edge: VertEdge,
}

impl Position {
    /// Widget name
    pub const KIND: &'static str = "Position";

    /// Create a new instance of a `Position`
    pub fn new(horz_edge: HorzEdge, vert_edge: VertEdge) -> Self {
        Self { child: None, horz_edge, vert_edge }
    }

    /// Position to the left
    pub fn left(&mut self, offset: i32) {
        self.horz_edge = HorzEdge::Left(offset);
    }

    /// Position to the right
    pub fn right(&mut self, offset: i32) {
        self.horz_edge = HorzEdge::Right(offset);
    }

    /// Position at the top
    pub fn top(&mut self, offset: i32) {
        self.vert_edge = VertEdge::Top(offset);
    }

    /// Position at the bottom
    pub fn bottom(&mut self, offset: i32) {
        self.vert_edge = VertEdge::Bottom(offset);
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(HorzEdge::Left(0), VertEdge::Top(0))
    }
}

impl Widget for Position {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        match self.child.as_mut() {
            Some(child) => {
                let mut size = child.layout(ctx.constraints, ctx.force_layout);
                if let HorzEdge::Right(_) = self.horz_edge {
                    size.width = ctx.constraints.max_width;
                }
                if let VertEdge::Bottom(_) = self.vert_edge {
                    size.height = ctx.constraints.max_height;
                }
                size
            }
            None => Size::ZERO,
        }
    }

    fn position(&mut self, mut ctx: PositionCtx) {
        let child = match self.child.as_mut() {
            Some(c) => c,
            None => return,
        };

        let x = match self.horz_edge {
            HorzEdge::Left(x) => x,
            HorzEdge::Right(x) => ctx.size.width as i32 - x - child.size.width as i32,
        };

        let y = match self.vert_edge {
            VertEdge::Top(y) => y,
            VertEdge::Bottom(y) => ctx.size.height as i32 - y - child.size.height as i32,
        };

        ctx.pos += Pos::new(x, y);
        child.position(ctx.pos);
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let child = match self.child.as_mut() {
            Some(c) => c,
            None => return,
        };

        child.paint(ctx.to_unsized());
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        match self.child.as_mut() {
            Some(c) => vec![c],
            None => vec![],
        }
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.child = Some(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(ref child) = self.child {
            if child.id.eq(child_id) {
                return self.child.take();
            }
        }
        None
    }

    fn update(&mut self, ctx: UpdateCtx) {
        for (k, _) in &ctx.attributes {
            match k.as_str() {
                fields::LEFT => self.horz_edge = HorzEdge::Left(ctx.attributes.left().unwrap_or(0)),
                fields::RIGHT => self.horz_edge = HorzEdge::Right(ctx.attributes.right().unwrap_or(0)),
                fields::TOP => self.vert_edge = VertEdge::Top(ctx.attributes.top().unwrap_or(0)),
                fields::BOTTOM => self.vert_edge = VertEdge::Bottom(ctx.attributes.bottom().unwrap_or(0)),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::test_widget;
    use crate::widgets::{Border, BorderStyle, Sides, Text};

    fn test_position(h_edge: HorzEdge, v_edge: VertEdge, expected: &str) {
        let mut root = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        let mut position = Position::new(h_edge, v_edge);
        position.add_child(Text::with_text("x").into_container(NodeId::auto()));
        root.add_child(position.into_container(NodeId::auto()));
        test_widget(root, expected);
    }

    #[test]
    fn top_left() {
        test_position(
            HorzEdge::Left(0),
            VertEdge::Top(0),
            r#"
            ┌───┐
            │x  │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn top_right() {
        test_position(
            HorzEdge::Right(0),
            VertEdge::Top(0),
            r#"
            ┌───┐
            │  x│
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn bottom_right() {
        test_position(
            HorzEdge::Right(0),
            VertEdge::Bottom(0),
            r#"
            ┌───┐
            │   │
            │  x│
            └───┘
            "#,
        );
    }

    #[test]
    fn bottom_left() {
        test_position(
            HorzEdge::Left(0),
            VertEdge::Bottom(0),
            r#"
            ┌───┐
            │   │
            │x  │
            └───┘
            "#,
        );
    }
}
