use anathema_render::Size;
use anathema_widget_core::contexts::{PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{AnyWidget, FactoryContext, LayoutNodes, Nodes, Widget, WidgetFactory};

use crate::layout::spacers::SpacerLayout;

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
#[derive(Debug, PartialEq)]
pub struct Spacer;

impl Spacer {
    /// Widget name
    pub const KIND: &'static str = "Spacer";
}

impl Widget for Spacer {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        SpacerLayout.layout(nodes)
    }

    fn position<'tpl>(&mut self, _children: &mut Nodes, _ctx: PositionCtx) {}

    fn paint(&mut self, _children: &mut Nodes, _ctx: PaintCtx<'_, WithSize>) {}
}

pub(crate) struct SpacerFactory;

impl WidgetFactory for SpacerFactory {
    fn make(&self, _ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        Ok(Box::new(Spacer))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    #[test]
    fn space_out_hstack() {
        let expr = expression(
            "border",
            None,
            [],
            [expression(
                "hstack",
                None,
                [],
                [
                    expression("text", Some("left".into()), [], []),
                    expression("spacer", None, [], []),
                    expression("text", Some("right".into()), [], []),
                ],
            )],
        );

        test_widget(
            expr,
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
        let expr = expression(
            "vstack",
            None,
            [],
            [
                expression("text", Some("top".into()), [], []),
                expression("spacer", None, [], []),
                expression("text", Some("bottom".into()), [], []),
            ],
        );

        test_widget(
            expr,
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
        let expr = expression(
            "vstack",
            None,
            [],
            [
                expression("text", Some("top".into()), [], []),
                expression("spacer", None, [], []),
                expression("text", Some("centre".into()), [], []),
                expression("spacer", None, [], []),
                expression("text", Some("bottom".into()), [], []),
            ],
        );

        test_widget(
            expr,
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
