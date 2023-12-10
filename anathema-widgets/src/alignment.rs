use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, Value, ValueExpr};
use anathema_widget_core::contexts::{LayoutCtx, PositionCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::nodes::Nodes;
use anathema_widget_core::layout::{Align, Layouts, Layout};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, Pos, Widget, WidgetContainer, WidgetFactory,
};

use crate::layout::single::Single;

/// Then `Alignment` widget "inflates" the parent to its maximum constraints
/// See [`Align`](crate::layout::Align) for more information.
///
/// If the alignment has no children it will have a size of zero.
#[derive(Debug)]
pub struct Alignment {
    /// The alignment
    pub alignment: Value<Align>,
}

impl Alignment {
    /// Alignment
    pub const KIND: &'static str = "Alignment";
}

impl Widget for Alignment {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        let size = Single.layout(nodes)?;
        if size == Size::ZERO {
            Ok(Size::ZERO)
        } else {
            let align = self.alignment.value_or_default();
            match align {
                Align::TopLeft => Ok(size),
                _ => Ok(nodes.constraints.expand_all(size)),
            }
        }
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.alignment.resolve(context, None);
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        if let Some((child, children)) = children.first_mut() {
            let width = ctx.inner_size.width as i32;
            let height = ctx.inner_size.height as i32;
            let child_width = child.outer_size().width as i32;
            let child_height = child.outer_size().height as i32;

            let child_offset = match self.alignment.value_or_default() {
                Align::TopLeft => Pos::ZERO,
                Align::Top => Pos::new(width / 2 - child_width / 2, 0),
                Align::TopRight => Pos::new(width - child_width, 0),
                Align::Right => Pos::new(width - child_width, height / 2 - child_height / 2),
                Align::BottomRight => Pos::new(width - child_width, height - child_height),
                Align::Bottom => Pos::new(width / 2 - child_width / 2, height - child_height),
                Align::BottomLeft => Pos::new(0, height - child_height),
                Align::Left => Pos::new(0, height / 2 - child_height / 2),
                Align::Centre => {
                    Pos::new(width / 2 - child_width / 2, height / 2 - child_height / 2)
                }
            };

            child.position(children, ctx.pos + child_offset);
        }
    }
}

pub(crate) struct AlignmentFactory;

impl WidgetFactory for AlignmentFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = Alignment {
            alignment: ctx.get("align"),
        };
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::layout::{Constraints, Padding};
    use anathema_widget_core::testing::{expression, FakeTerm};

    use super::*;
    use crate::testing::test_widget;

    fn align_widget(align: Align, expected: FakeTerm) {
        let text = expression("text", Some("AB"), [], []);
        let alignment = expression(
            "alignment",
            None,
            [("align".into(), ValueExpr::String(align.to_string().into()))],
            [text],
        );
        test_widget(alignment, expected);
    }

    #[test]
    fn align_top_left() {
        align_widget(
            Align::TopLeft,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║AB              ║
            ║                ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_top() {
        align_widget(
            Align::Top,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║         AB         ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_top_right() {
        align_widget(
            Align::TopRight,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║              AB║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_right() {
        align_widget(
            Align::Right,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║              AB║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_bottom_right() {
        align_widget(
            Align::BottomRight,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║                ║
            ║              AB║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_bottom() {
        align_widget(
            Align::Bottom,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║                ║
            ║       AB       ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_bottom_left() {
        align_widget(
            Align::BottomLeft,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║                ║
            ║AB              ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_left() {
        align_widget(
            Align::Left,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║AB              ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn align_centre() {
        align_widget(
            Align::Centre,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║       AB       ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn unconstrained_alignment_without_child() {
        // let constraints = Constraints::unbounded();
        // let mut children = vec![];
        // let data = DataCtx::default();
        // let store = Values::new(&data);
        // let ctx = LayoutCtx::new(&[], &store, constraints, Padding::ZERO);
        // let mut alignment = Alignment::new(Align::Left);
        // let actual = alignment.layout(ctx, &mut children).unwrap();
        // let expected = Size::ZERO;
        // assert_eq!(expected, actual);
    }
}
