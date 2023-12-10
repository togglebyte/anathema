use anathema_render::{Size, Style};
use anathema_values::{Attributes, Context, NodeId, Value, ValueExpr};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Layout, Layouts};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, LocalPos, Nodes, Widget, WidgetContainer,
    WidgetFactory, WidgetStyle,
};

use crate::layout::single::Single;

const DEFAULT_FACTOR: usize = 1;

/// The `Expand` widget will fill up all remaining space inside a widget in both horizontal and
/// vertical direction.
///
/// To only expand in one direction, set the `direction` of the `Expand` widget.
///
/// A [`Direction`] can be set when creating a new widget
///
/// The total available space is divided between the `Expand` widgets and multiplied by the
/// widgets `factor`.
///
/// ```ignore
/// # use anathema_widgets::{NodeId, HStack, Constraints, Widget};
/// use anathema_widgets::Expand;
/// let left = Expand::new(2, None, None);
/// let right = Expand::new(3, None, None);
/// # let left = left.into_container(NodeId::anon());
/// # let right = right.into_container(NodeId::anon());
/// # let left_id = left.id();
/// # let right_id = right.id();
///
/// // ... layout
///
/// # let mut root = HStack::new(10, 5);
/// # root.children.push(left);
/// # root.children.push(right);
/// # let mut root = root.into_container(NodeId::anon());
/// # root.layout(Constraints::new(10, 5), false);
/// # {
/// // The left `Expand` widget has a factor of two.
/// // The right `Expand` widget has a factor of three.
/// // Given the total width of ten, and a total factor count of five,
/// // This means the left widget has a width of four: `10 / 5 * 2`
/// // and the right widget has a width of six: `10 / 5 * 3`
///
/// let left = root.by_id(&left_id).unwrap();
/// assert_eq!(left.size().width, 4);
/// # }
///
/// let right = root.by_id(&right_id).unwrap();
/// assert_eq!(right.size().width, 6);
/// ```
#[derive(Debug)]
pub struct Expand {
    /// The direction to expand in.
    pub axis: Value<Axis>,
    /// Fill the space by repeating the characters.
    pub fill: Value<String>,
    /// The style of the expansion.
    pub style: WidgetStyle,
    pub(crate) factor: Value<usize>,
}

impl Expand {
    /// Widget name.
    pub const KIND: &'static str = "Expand";

    // /// Create a new instance of an `Expand` widget.
    // pub fn new(
    //     factor: impl Into<Option<usize>>,
    //     direction: impl Into<Option<Axis>>,
    //     fill: impl Into<Option<String>>,
    // ) -> Self {
    //     let factor = factor.into();
    //     let axis = direction.into();

    //     Self {
    //         factor: factor.value_or(DEFAULT_FACTOR),
    //         axis,
    //         fill: fill.into().unwrap_or(String::new()),
    //         style: Style::new(),
    //     }
    // }
}

impl Widget for Expand {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        let mut size = Single.layout(nodes)?;

        match self.axis.value_ref() {
            Some(Axis::Horizontal) => size.width = nodes.constraints.max_width,
            Some(Axis::Vertical) => size.height = nodes.constraints.max_height,
            None => {
                size.width = nodes.constraints.max_width;
                size.height = nodes.constraints.max_height;
            }
        }

        Ok(size)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        if let Some((widget, children)) = children.first_mut() {
            widget.position(children, ctx.pos)
        }
    }

    fn paint(&mut self, children: &mut Nodes, mut ctx: PaintCtx<'_, WithSize>) {
        if let Some(fill) = self.fill.value_ref() {
            for y in 0..ctx.local_size.height {
                let mut used_width = 0;
                loop {
                    let pos = LocalPos::new(used_width, y);
                    let Some(p) = ctx.print(fill, self.style.style(), pos) else {
                        break;
                    };
                    used_width += p.x - used_width;
                }
            }
        }

        if let Some((widget, children)) = children.first_mut() {
            let ctx = ctx.sub_context(None);
            widget.paint(children, ctx);
        }
    }
}

pub(crate) struct ExpandFactory;

impl WidgetFactory for ExpandFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = Expand {
            axis: ctx.get("axis"),
            fill: ctx.get("fill"),
            factor: ctx.get("factor"),
            style: ctx.style(),
        };

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::testing::{expression, FakeTerm};

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn expand_border() {
        // let border = Border::thin(None, None);
        // let body = [template("expand", (), vec![])];

        let border = expression("border", None, [], [expression("expand", None, [], [])]);

        test_widget(
            border,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─────────────┐║
            ║│             │║
            ║│             │║
            ║│             │║
            ║│             │║
            ║└─────────────┘║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expand_horz_with_factors() {
        let hstack = expression(
            "hstack",
            None,
            [],
            [
                expression(
                    "border",
                    None,
                    [],
                    [expression(
                        "expand",
                        None,
                        [("factor".into(), 1.into())],
                        [],
                    )],
                ),
                expression(
                    "border",
                    None,
                    [],
                    [expression(
                        "expand",
                        None,
                        [("factor".into(), 2.into())],
                        [],
                    )],
                ),
            ],
        );

        test_widget(
            hstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌───┐┌────────┐║
            ║│   ││        │║
            ║│   ││        │║
            ║│   ││        │║
            ║│   ││        │║
            ║└───┘└────────┘║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expand_vert_with_factors() {
        let vstack = expression(
            "vstack",
            None,
            [],
            [
                expression(
                    "border",
                    None,
                    [],
                    [expression(
                        "expand",
                        None,
                        [("factor".into(), 1.into())],
                        [],
                    )],
                ),
                expression(
                    "border",
                    None,
                    [],
                    [expression(
                        "expand",
                        None,
                        [("factor".into(), 2.into())],
                        [],
                    )],
                ),
            ],
        );

        test_widget(
            vstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─────────────┐║
            ║│             │║
            ║└─────────────┘║
            ║┌─────────────┐║
            ║│             │║
            ║│             │║
            ║│             │║
            ║│             │║
            ║└─────────────┘║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    // #[test]
    // fn expand_horz() {
    //     let border = Border::thin(None, None);
    //     let body = [template(
    //         "expand",
    //         [("axis", Axis::Horizontal)],
    //         vec![template_text("A cup of tea please")],
    //     )];
    //     test_widget(
    //         border,
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [════════════════╗
    //         ║┌────────────────────────────┐║
    //         ║│A cup of tea please         │║
    //         ║└────────────────────────────┘║
    //         ║                              ║
    //         ║                              ║
    //         ╚══════════════════════════════╝
    //         "#,
    //         ),
    //     );
    // }

    // #[test]
    // fn expand_vert() {
    //     let border = Border::thin(None, None);
    //     let body = [template(
    //         "expand",
    //         [("axis", Axis::Vertical)],
    //         vec![template_text("A cup of tea please")],
    //     )];
    //     test_widget(
    //         border,
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [════════════════╗
    //         ║┌───────────────────┐         ║
    //         ║│A cup of tea please│         ║
    //         ║│                   │         ║
    //         ║│                   │         ║
    //         ║│                   │         ║
    //         ║│                   │         ║
    //         ║└───────────────────┘         ║
    //         ╚══════════════════════════════╝
    //         "#,
    //         ),
    //     );
    // }

    // #[test]
    // fn expand_all() {
    //     let border = Border::thin(None, None);
    //     let body = [template(
    //         "expand",
    //         (),
    //         vec![template_text("A cup of tea please")],
    //     )];
    //     test_widget(
    //         border,
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [════════════════╗
    //         ║┌────────────────────────────┐║
    //         ║│A cup of tea please         │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║└────────────────────────────┘║
    //         ╚══════════════════════════════╝
    //         "#,
    //         ),
    //     );
    // }

    // #[test]
    // fn expand_with_padding() {
    //     let border = Border::thin(None, None);
    //     let body = [template(
    //         "expand",
    //         [("padding", 1)],
    //         vec![template_text("A cup of tea please")],
    //     )];
    //     test_widget(
    //         border,
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [════════════════╗
    //         ║┌────────────────────────────┐║
    //         ║│                            │║
    //         ║│ A cup of tea please        │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║└────────────────────────────┘║
    //         ╚══════════════════════════════╝
    //         "#,
    //         ),
    //     );
    // }

    // #[test]
    // fn expanding_inside_vstack() {
    //     let vstack = VStack::new(None, None);
    //     let body = [
    //         template(
    //             "border",
    //             (),
    //             [template(
    //                 "hstack",
    //                 (),
    //                 [
    //                     template_text("A cup of tea please"),
    //                     template("spacer", (), []),
    //                 ],
    //             )],
    //         ),
    //         template(
    //             "expand",
    //             (),
    //             [template(
    //                 "border",
    //                 (),
    //                 [template("expand", (), [template_text("Hello world")])],
    //             )],
    //         ),
    //     ];

    //     test_widget(
    //         vstack,
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [════════════════╗
    //         ║┌────────────────────────────┐║
    //         ║│A cup of tea please         │║
    //         ║└────────────────────────────┘║
    //         ║┌────────────────────────────┐║
    //         ║│Hello world                 │║
    //         ║│                            │║
    //         ║│                            │║
    //         ║└────────────────────────────┘║
    //         ╚══════════════════════════════╝
    //         "#,
    //         ),
    //     );
    // }
}
