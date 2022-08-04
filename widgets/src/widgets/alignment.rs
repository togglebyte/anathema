use display::Size;

use crate::attributes::{fields, Attributes};
use crate::layout::Align;
use crate::Pos;

use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// Then `Alignment` widget "inflates" the parent to its maximum constraints
/// See [`Align`](crate::layout::Align) for more information.
///
/// ```
/// use widgets::{Align, Alignment};
/// let alignment = Alignment::new(Align::TopRight);
/// ```
#[derive(Debug)]
pub struct Alignment {
    /// The inner widget which will be aligned
    pub child: Option<WidgetContainer>,
    /// The alignment
    pub alignment: Align,
}

impl Alignment {
    /// Alignment
    pub const KIND: &'static str = "Alignment";

    /// Create a new instance of an `Alignment` widget
    pub fn new(alignment: Align) -> Self {
        Self { child: None, alignment }
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

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        // -----------------------------------------------------------------------------
        //     - Layout child -
        // -----------------------------------------------------------------------------
        match self.child.as_mut() {
            Some(child) => {
                let _child_size = child.layout(ctx.constraints, ctx.force_layout);
                Size::new(ctx.constraints.max_width, ctx.constraints.max_height)
            }
            None => Size::ZERO,
        }
    }

    fn position(&mut self, ctx: PositionCtx) {
        if let Some(child) = self.child.as_mut() {
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

            child.position(ctx.pos + child_offset);
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        if let Some(child) = self.child.as_mut() {
            let ctx = ctx.to_unsized();
            child.paint(ctx);
        }
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

    fn update(&mut self, attributes: Attributes) {
        attributes.has(fields::ALIGNMENT).then(|| self.alignment = attributes.alignment().unwrap_or(Align::Left));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::layout::Constraints;
    use crate::testing::test_widget;
    use crate::{Border, BorderStyle, Sides, Text, Padding};
    

    fn align_widget(align: Align, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        let mut alignment = Alignment::new(align);
        alignment.child = Some(Text::with_text("xx").into_container(NodeId::auto()));
        border.child = Some(alignment.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn align_top_left() {
        align_widget(
            Align::TopLeft,
            r#"
            ┌────────┐
            │xx      │
            │        │
            │        │
            │        │
            └────────┘
        "#,
        );
    }

    #[test]
    fn align_top() {
        align_widget(
            Align::Top,
            r#"
            ┌────────┐
            │   xx   │
            │        │
            │        │
            │        │
            └────────┘
        "#,
        );
    }

    #[test]
    fn align_top_right() {
        align_widget(
            Align::TopRight,
            r#"
            ┌────────┐
            │      xx│
            │        │
            │        │
            │        │
            └────────┘
        "#,
        );
    }

    #[test]
    fn align_right() {
        align_widget(
            Align::Right,
            r#"
            ┌──────┐
            │      │
            │    xx│
            │      │
            └──────┘
        "#,
        );
    }

    #[test]
    fn align_bottom_right() {
        align_widget(
            Align::BottomRight,
            r#"
            ┌──────┐
            │      │
            │      │
            │    xx│
            └──────┘
        "#,
        );
    }

    #[test]
    fn align_bottom() {
        align_widget(
            Align::Bottom,
            r#"
            ┌──────┐
            │      │
            │      │
            │  xx  │
            └──────┘
        "#,
        );
    }

    #[test]
    fn align_bottom_left() {
        align_widget(
            Align::BottomLeft,
            r#"
            ┌──────┐
            │      │
            │      │
            │xx    │
            └──────┘
        "#,
        );
    }

    #[test]
    fn align_left() {
        align_widget(
            Align::Left,
            r#"
            ┌──────┐
            │      │
            │xx    │
            │      │
            └──────┘
        "#,
        );
    }

    #[test]
    fn align_centre() {
        align_widget(
            Align::Centre,
            r#"
            ┌──────┐
            │      │
            │  xx  │
            │      │
            └──────┘
        "#,
        );
    }

    #[test]
    fn unconstrained_alignment_without_child() {
        let mut alignment = Alignment::default();
        let constraints = Constraints::unbounded();
        let actual = alignment.layout(LayoutCtx::new(constraints, false, Padding::ZERO));
        let expected = Size::ZERO;
        assert_eq!(expected, actual);
    }
}
