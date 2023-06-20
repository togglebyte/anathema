use anathema_render::Size;

use super::{NodeId, PaintCtx, PositionCtx, Widget, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{WidgetContainer, TextPath, AnyWidget};
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;

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

    fn layout(&mut self, mut ctx: LayoutCtx<'_, '_, '_>) -> Result<Size> {
        debug_assert!(
            ctx.constraints.is_width_tight() && ctx.constraints.is_height_tight(),
            "the layout context needs to be tight for a spacer"
        );

        Ok(Size::new(ctx.constraints.min_width, ctx.constraints.min_height))
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {}

    fn paint<'gen, 'ctx>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
    }

    //     // fn update(&mut self, _: UpdateCtx) {}
}

pub(crate) struct SpacerFactory;

impl WidgetFactory for SpacerFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        Ok(Box::new(Spacer))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Attributes, HStack, VStack};
    use crate::template::{template, template_text, Template};
    use crate::testing::{test_widget, FakeTerm};

    #[test]
    fn space_out_hstack() {
        let hstack = HStack::new(None, None);
        let body = [
            template_text("left"),
            template("spacer", (), vec![]),
            template_text("right"),
        ];
        test_widget(
            hstack,
            &body,
            FakeTerm::from_str(
            r#"
            ╔═] Fake term [═╗
            ║left      right║
            ║               ║
            ║               ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            )
        );
    }

}
