use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::{LayoutCtx, PositionCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Align, Layouts};
use anathema_widget_core::{AnyWidget, Nodes, Pos, Widget, WidgetContainer, WidgetFactory};

use crate::layout::single::Single;

/// Then `Alignment` widget "inflates" the parent to its maximum constraints
/// See [`Align`](crate::layout::Align) for more information.
///
/// If the alignment has no children it will be zero sized.
///
/// ```
/// use anathema_widget_core::layout::Align;
/// use anathema_widgets::Alignment;
/// let alignment = Alignment::new(Align::TopRight);
/// ```
pub struct Alignment {
    /// The alignment
    pub alignment: Align,
}

impl Alignment {
    /// Alignment
    pub const KIND: &'static str = "Alignment";

    /// Create a new instance of an `Alignment` widget
    pub fn new(alignment: Align) -> Self {
        Self { alignment }
    }
}

impl Widget for Alignment {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, children: &mut Nodes, mut layout: LayoutCtx, data: Context<'_, '_>) -> Result<Size> {
        panic!()
        // let mut layout = Layouts::new(Single, &mut ctx);
        // layout.layout(children, data)?;
        // let size = layout.size()?;
        // if size == Size::ZERO {
        //     Ok(Size::ZERO)
        // } else {
        //     layout.expand_horz().expand_vert().size()
        // }
    }

    fn position(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        if let Some((child, children)) = children.first_mut() {
            let width = ctx.inner_size.width as i32;
            let height = ctx.inner_size.height as i32;
            let child_width = child.outer_size().width as i32;
            let child_height = child.outer_size().height as i32;

            let child_offset = match self.alignment {
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

// Frame 1
// Constraint = 100
// list_a = [1, 2, 3]
// list_b = [1, 2, 3]
//
// Template:
//
// border
//    vstack  height 3
//        for x in list_a
//            text x
//
//    vstack height 3
//        for x in list_b
//            text x
//

// Frame 2
// Constraint = 100
// list_a = [1, 2, 3]
// list_b = [1, 2, 4] <- only thing that has changed
//
// Template:
//
// border
//    vstack  height 3
//        for x in list_a
//            text x
//
//    vstack height 3
//        for x in list_b
//            text x
//


pub(crate) struct AlignmentFactory;

impl WidgetFactory for AlignmentFactory {
    fn make(&self, data: Context<'_, '_>) -> Result<Box<dyn AnyWidget>> {
        let alignment: Align = data.get(&"align".into()).unwrap_or(Align::TopLeft); // Cached::new_or("align", &data, Align::TopLeft);
        let widget = Alignment::new(alignment);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::contexts::DataCtx;
    use anathema_widget_core::layout::{Constraints, Padding};
    use anathema_widget_core::template::template_text;
    use anathema_widget_core::testing::FakeTerm;
    use anathema_widget_core::Values;

    use super::*;
    use crate::testing::test_widget;

    fn align_widget(align: Align, expected: FakeTerm) {
        let text = template_text("AB");
        let alignment = Alignment::new(align);
        let body = [text];
        test_widget(alignment, body, expected);
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
        let constraints = Constraints::unbounded();
        let mut children = vec![];
        let data = DataCtx::default();
        let store = Values::new(&data);
        let ctx = LayoutCtx::new(&[], &store, constraints, Padding::ZERO);
        let mut alignment = Alignment::new(Align::Left);
        let actual = alignment.layout(ctx, &mut children).unwrap();
        let expected = Size::ZERO;
        assert_eq!(expected, actual);
    }
}
