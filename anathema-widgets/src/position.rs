use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::PositionCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{HorzEdge, Layout, VertEdge};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, Nodes, Pos, Widget, WidgetFactory,
};

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

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        match &mut self.horz_edge {
            HorzEdge::Left(val) => val.resolve(context, node_id),
            HorzEdge::Right(val) => val.resolve(context, node_id),
        }
        match &mut self.vert_edge {
            VertEdge::Top(val) => val.resolve(context, node_id),
            VertEdge::Bottom(val) => val.resolve(context, node_id),
        }
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
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

    fn position<'tpl>(&mut self, children: &mut Nodes<'_>, mut ctx: PositionCtx) {
        let (child, children) = match children.first_mut() {
            Some(c) => c,
            None => return,
        };

        let x = match &self.horz_edge {
            HorzEdge::Left(x) => x.value_or(0),
            HorzEdge::Right(x) => {
                ctx.inner_size.width as i32 - x.value_or(0) - child.size.width as i32
            }
        };

        let y = match &self.vert_edge {
            VertEdge::Top(y) => y.value_or(0),
            VertEdge::Bottom(y) => {
                ctx.inner_size.height as i32 - y.value_or(0) - child.size.height as i32
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
            },
            val => HorzEdge::Left(val),
        };

        let vert_edge = match ctx.get("top") {
            Value::Empty => match ctx.get("bottom") {
                Value::Empty => VertEdge::Top(Value::Static(0)),
                val => VertEdge::Bottom(val),
            },
            val => VertEdge::Top(val),
        };

        let widget = Position::new(horz_edge, vert_edge);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    #[test]
    fn top_left() {
        let expr = expression(
            "position",
            None,
            [
                ("left".to_string(), 0.into()),
                ("top".to_string(), 0.into()),
            ],
            [expression("text", Some("top left".into()), [], [])],
        );

        test_widget(
            expr,
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
        let expr = expression(
            "position",
            None,
            [
                ("right".to_string(), 0.into()),
                ("top".to_string(), 0.into()),
            ],
            [expression("text", Some("top right".into()), [], [])],
        );

        test_widget(
            expr,
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
        let expr = expression(
            "position",
            None,
            [
                ("right".to_string(), 0.into()),
                ("bottom".to_string(), 0.into()),
            ],
            [expression("text", Some("bottom right".into()), [], [])],
        );
        test_widget(
            expr,
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
        let expr = expression(
            "position",
            None,
            [
                ("left".to_string(), 0.into()),
                ("bottom".to_string(), 0.into()),
            ],
            [expression("text", Some("bottom left".into()), [], [])],
        );
        test_widget(
            expr,
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
