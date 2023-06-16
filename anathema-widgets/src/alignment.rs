use anathema_render::Size;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::layout::Layouts;
use crate::layout::single::Single;
use crate::{Align, Widget, Pos, PositionCtx, WidgetContainer};

/// Then `Alignment` widget "inflates" the parent to its maximum constraints
/// See [`Align`](crate::layout::Align) for more information.
///
/// ```
/// use anathema_widgets::{Align, Alignment};
/// let alignment = Alignment::new(Align::TopRight);
/// ```
#[derive(Debug)]
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

impl Default for Alignment {
    fn default() -> Self {
        Self::new(Align::Left)
    }
}

impl Widget for Alignment {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, mut ctx: LayoutCtx<'_, '_, '_>) -> Result<Size> {
        // TODO: should alignment ALWAYS expand the size, even if there is no child widget?
        Layouts::new(Single, &mut ctx).layout()?
            .expand_horz()
            .expand_vert()
            .size()
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        let mut pos = ctx.padded_position();
            if let Some(child) = children.first_mut() {
                let alignment = self.alignment;

                let width = ctx.size.width as i32;
                let height = ctx.size.height as i32;
                let child_width = child.size.width as i32;
                let child_height = child.size.height as i32;

                let child_offset = match alignment {
                    Align::TopLeft => Pos::ZERO,
                    Align::Top => Pos::new(width / 2 - child_width / 2, 0),
                    Align::TopRight => Pos::new(width - child_width, 0),
                    Align::Right => Pos::new(width - child_width, height / 2 - child_height / 2),
                    Align::BottomRight => Pos::new(width - child_width, height - child_height),
                    Align::Bottom => Pos::new(width / 2 - child_width / 2, height - child_height),
                    Align::BottomLeft => Pos::new(0, height - child_height),
                    Align::Left => Pos::new(0, height / 2 - child_height / 2),
                    Align::Centre => Pos::new(width / 2 - child_width / 2, height / 2 - child_height / 2),
                };

                child.position(ctx.padded_position() + child_offset);
            }
        }

    //     // fn update(&mut self, ctx: UpdateCtx) {
    //     //     ctx.attributes
    //     //         .has(fields::ALIGNMENT)
    //     //         .then(|| self.alignment = ctx.attributes.alignment().unwrap_or(Align::Left));
    //     // }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::testing::test_widget;
    // use crate::Constraints;
    // use crate::{Border, BorderStyle, Padding, Sides, Text};

    // fn align_widget(align: Align, expected: &str) {
    //     let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
    //     let mut alignment = Alignment::new(align);
    //     alignment.child = Some(Text::with_text("xx").into_container(NodeId::anon()));
    //     border.child = Some(alignment.into_container(NodeId::anon()));
    //     test_widget(border, expected);
    // }

    // #[test]
    // fn align_top_left() {
    //     align_widget(
    //         Align::TopLeft,
    //         r#"
    //         ┌────────┐
    //         │xx      │
    //         │        │
    //         │        │
    //         │        │
    //         └────────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_top() {
    //     align_widget(
    //         Align::Top,
    //         r#"
    //         ┌────────┐
    //         │   xx   │
    //         │        │
    //         │        │
    //         │        │
    //         └────────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_top_right() {
    //     align_widget(
    //         Align::TopRight,
    //         r#"
    //         ┌────────┐
    //         │      xx│
    //         │        │
    //         │        │
    //         │        │
    //         └────────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_right() {
    //     align_widget(
    //         Align::Right,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │    xx│
    //         │      │
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_bottom_right() {
    //     align_widget(
    //         Align::BottomRight,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │      │
    //         │    xx│
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_bottom() {
    //     align_widget(
    //         Align::Bottom,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │      │
    //         │  xx  │
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_bottom_left() {
    //     align_widget(
    //         Align::BottomLeft,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │      │
    //         │xx    │
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_left() {
    //     align_widget(
    //         Align::Left,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │xx    │
    //         │      │
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn align_centre() {
    //     align_widget(
    //         Align::Centre,
    //         r#"
    //         ┌──────┐
    //         │      │
    //         │  xx  │
    //         │      │
    //         └──────┘
    //     "#,
    //     );
    // }

    // #[test]
    // fn unconstrained_alignment_without_child() {
    //     let mut alignment = Alignment::default();
    //     let constraints = Constraints::unbounded();
    //     let actual = alignment.layout(LayoutCtx::new(constraints, false, Padding::ZERO));
    //     let expected = Size::ZERO;
    //     assert_eq!(expected, actual);
    // }
}
