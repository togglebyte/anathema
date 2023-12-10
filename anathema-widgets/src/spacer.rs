use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, ValueExpr};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Layouts, Layout};
use anathema_widget_core::{
    AnyWidget, FactoryContext, Nodes, Widget, WidgetContainer, WidgetFactory, LayoutNodes,
};

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

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        // debug_assert!(
        //     layout.constraints.is_width_tight() && layout.constraints.is_height_tight(),
        //     "the layout context needs to be tight for a spacer"
        // );

        SpacerLayout.layout(nodes)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {}

    fn paint(&mut self, children: &mut Nodes, mut ctx: PaintCtx<'_, WithSize>) {}
}

pub(crate) struct SpacerFactory;

impl WidgetFactory for SpacerFactory {
    fn make(&self, _ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        Ok(Box::new(Spacer))
    }
}

// #[cfg(test)]
// mod test {

//     use anathema_widget_core::template::{template, template_text};
//     use anathema_widget_core::testing::FakeTerm;

//     use crate::testing::test_widget;
//     use crate::{Border, VStack};

//     #[test]
//     fn space_out_hstack() {
//         let border = Border::thin(None, None);
//         let body = [template(
//             "hstack",
//             (),
//             [
//                 template_text("left"),
//                 template("spacer", (), vec![]),
//                 template_text("right"),
//             ],
//         )];
//         test_widget(
//             border,
//             body,
//             FakeTerm::from_str(
//                 r#"
//             ╔═] Fake term [═╗
//             ║┌─────────────┐║
//             ║│left    right│║
//             ║└─────────────┘║
//             ║               ║
//             ║               ║
//             ║               ║
//             ║               ║
//             ╚═══════════════╝
//             "#,
//             ),
//         );
//     }

//     #[test]
//     fn space_out_vstack() {
//         let hstack = VStack::new(None, None);
//         let body = [
//             template_text("top"),
//             template("spacer", (), vec![]),
//             template_text("bottom"),
//         ];
//         test_widget(
//             hstack,
//             body,
//             FakeTerm::from_str(
//                 r#"
//             ╔═] Fake term [═╗
//             ║top            ║
//             ║               ║
//             ║               ║
//             ║               ║
//             ║               ║
//             ║bottom         ║
//             ╚═══════════════╝
//             "#,
//             ),
//         );
//     }

//     #[test]
//     fn centre_using_spacers() {
//         let hstack = VStack::new(None, None);
//         let body = [
//             template_text("top"),
//             template("spacer", (), vec![]),
//             template_text("centre"),
//             template("spacer", (), vec![]),
//             template_text("bottom"),
//         ];
//         test_widget(
//             hstack,
//             body,
//             FakeTerm::from_str(
//                 r#"
//             ╔═] Fake term [═╗
//             ║top            ║
//             ║               ║
//             ║               ║
//             ║centre         ║
//             ║               ║
//             ║               ║
//             ║bottom         ║
//             ╚═══════════════╝
//             "#,
//             ),
//         );
//     }
// }
