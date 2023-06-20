use anathema_render::Size;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::layout::single::Single;
use crate::layout::Layouts;
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{
    Align, AnyWidget, PaintCtx, Pos, PositionCtx, TextPath, Widget, WidgetContainer, WithSize,
};

/// Then `Alignment` widget "inflates" the parent to its maximum constraints
/// See [`Align`](crate::layout::Align) for more information.
///
/// If the alignment has no children it will be zero sized.
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
        let mut layout = Layouts::new(Single, &mut ctx);
        layout.layout()?;
        let size = layout.size()?;
        if size == Size::ZERO {
            Ok(Size::ZERO)
        } else {
            layout.expand_horz().expand_vert().size()
        }
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        let mut pos = ctx.pos;
        if let Some(child) = children.first_mut() {
            let alignment = self.alignment;

            let width = ctx.inner_size.width as i32;
            let height = ctx.inner_size.height as i32;
            let child_width = child.outer_size().width as i32;
            let child_height = child.outer_size().height as i32;

            let child_offset = match alignment {
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

            child.position(ctx.pos + child_offset);
        }
    }

    //     // fn update(&mut self, ctx: UpdateCtx) {
    //     //     ctx.attributes
    //     //         .has(fields::ALIGNMENT)
    //     //         .then(|| self.alignment = ctx.attributes.alignment().unwrap_or(Align::Left));
    //     // }
}

pub(crate) struct AlignmentFactory;

impl WidgetFactory for AlignmentFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let align = values.alignment().unwrap_or(Align::TopLeft);
        let widget = Alignment::new(align);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gen::store::Store;
    use crate::template::{Template, template_text};
    use crate::testing::{test_widget, FakeTerm};
    use crate::{Attributes, Constraints, DataCtx, Lookup, Padding};

    fn align_widget(align: Align, expected: FakeTerm) {
        let text = template_text("AB");

        let alignment = Alignment::new(align);
        let children = [text];
        let widget = WidgetContainer::new(Box::new(alignment), &children);
        test_widget(widget, expected);
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
        let lookup = Lookup::default();
        let mut children = vec![];
        let data = DataCtx::default();
        let store = Store::new(&data);
        let ctx = LayoutCtx::new(
            &[],
            &store,
            constraints,
            Padding::ZERO,
            &mut children,
            &lookup,
        );
        let mut alignment = Alignment::default();
        let actual = alignment.layout(ctx).unwrap();
        let expected = Size::ZERO;
        assert_eq!(expected, actual);
    }
}
