use anathema_render::Size;

use super::{PaintCtx, PositionCtx, Widget, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{AnyWidget, TextPath, WidgetContainer};

/// Expand to fill in all available space.
///
/// Unlike the [`Expanded`](crate::Expanded) widget, the `Spacer` only works inside
/// [`HStack`](crate::HStack) and [`VStack`](crate::VStack), and flows in the
/// direction of the stack.
///
/// In an `HStack` the spacer will always expand to have the same height as the child with the most
/// height.
///
/// In an `VStack` the spacer will always expand to have the same width as the child with the most
/// width.
#[derive(Debug)]
pub struct Spacer;

impl Spacer {
    /// Widget name
    pub const KIND: &'static str = "Spacer";
}

impl Widget for Spacer {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, ctx: LayoutCtx<'_, '_, '_>, children: &mut Vec<WidgetContainer<'_>>) -> Result<Size> {
        // debug_assert!(
        //     ctx.constraints.is_width_tight() && ctx.constraints.is_height_tight(),
        //     "the layout context needs to be tight for a spacer"
        // );

        Ok(Size::new(
            ctx.constraints.min_width,
            ctx.constraints.min_height,
        ))
    }

    fn position<'gen, 'ctx>(&mut self, _: PositionCtx, _: &mut [WidgetContainer<'gen>]) {}

    fn paint<'gen, 'ctx>(&mut self, _: PaintCtx<'_, WithSize>, _: &mut [WidgetContainer<'gen>]) {}
}

pub(crate) struct SpacerFactory;

impl WidgetFactory for SpacerFactory {
    fn make(
        &self,
        _: ValuesAttributes<'_, '_>,
        _: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        Ok(Box::new(Spacer))
    }
}

#[cfg(test)]
mod test {

    use crate::template::{template, template_text};
    use crate::testing::{test_widget, FakeTerm};
    use crate::{HStack, VStack, Border};

    #[test]
    fn space_out_hstack() {
        let border = Border::thin(None, None);
        let body = [
            template("hstack", (), [
                template_text("left"),
                template("spacer", (), vec![]),
                template_text("right"),
            ])
        ];
        test_widget(
            border,
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─────────────┐║
            ║│left    right│║
            ║└─────────────┘║
            ║               ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn space_out_vstack() {
        let hstack = VStack::new(None, None);
        let body = [
            template_text("top"),
            template("spacer", (), vec![]),
            template_text("bottom"),
        ];
        test_widget(
            hstack,
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║top            ║
            ║               ║
            ║               ║
            ║               ║
            ║               ║
            ║bottom         ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn centre_using_spacers() {
        let hstack = VStack::new(None, None);
        let body = [
            template_text("top"),
            template("spacer", (), vec![]),
            template_text("centre"),
            template("spacer", (), vec![]),
            template_text("bottom"),
        ];
        test_widget(
            hstack,
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║top            ║
            ║               ║
            ║               ║
            ║centre         ║
            ║               ║
            ║               ║
            ║bottom         ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
