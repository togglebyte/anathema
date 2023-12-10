use anathema_render::Size;
use anathema_values::{Context, NodeId, ValueRef, Value};
use anathema_widget_core::contexts::{LayoutCtx, PositionCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{HorzEdge, Layouts, VertEdge, Layout};
use anathema_widget_core::{AnyWidget, Nodes, Pos, Widget, WidgetContainer, WidgetFactory, FactoryContext, LayoutNodes};

use crate::layout::single::Single;

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
        Self {
            horz_edge,
            vert_edge,
        }
    }
}

impl Widget for Position {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        let mut layout = Single;
        let mut size = layout.layout(nodes)?;

        if let HorzEdge::Right(_) = self.horz_edge {
            size = nodes.constraints.expand_horz(size);
        }
        if let VertEdge::Bottom(_) = self.vert_edge {
            size = nodes.constraints.expand_vert(size);
        }

        Ok(size)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, mut ctx: PositionCtx) {
        let (child, children) = match children.first_mut() {
            Some(c) => c,
            None => return,
        };

        let x = match &self.horz_edge {
            HorzEdge::Left(x) => x.value_or(0),
            HorzEdge::Right(x) => ctx.inner_size.width as i32 - x.value_or(0) - child.outer_size().width as i32,
        };

        let y = match &self.vert_edge {
            VertEdge::Top(y) => y.value_or(0),
            VertEdge::Bottom(y) => {
                ctx.inner_size.height as i32 - y.value_or(0) - child.outer_size().height as i32
            }
        };

        ctx.pos += Pos::new(x, y);
        child.position(children, ctx.pos);
    }
}

pub(crate) struct PositionFactory;

impl WidgetFactory for PositionFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let horz_edge = match ctx.get("left") {
            Value::Empty => match ctx.get("right") {
                Value::Empty => HorzEdge::Right(Value::Static(0)),
                val => HorzEdge::Right(val),
            }
            val => HorzEdge::Left(val),
        };

        let vert_edge = match ctx.get("top") {
            Value::Empty => match ctx.get("bottom") {
                Value::Empty => VertEdge::Top(Value::Static(0)),
                val => VertEdge::Bottom(val),
            }
            val => VertEdge::Top(val),
        };

        let widget = Position::new(horz_edge, vert_edge);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::template_text;
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn top_left() {
        let body = [template_text("top left")];

        test_widget(
            Position::new(HorzEdge::Left(0), VertEdge::Top(0)),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║top left       ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn top_right() {
        let body = [template_text("top right")];
        test_widget(
            Position::new(HorzEdge::Right(0), VertEdge::Top(0)),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║      top right║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn bottom_right() {
        let body = [template_text("bottom right")];
        test_widget(
            Position::new(HorzEdge::Right(0), VertEdge::Bottom(0)),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║               ║
            ║               ║
            ║               ║
            ║   bottom right║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn bottom_left() {
        let body = [template_text("bottom left")];
        test_widget(
            Position::new(HorzEdge::Left(0), VertEdge::Bottom(0)),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║               ║
            ║               ║
            ║               ║
            ║bottom left    ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
