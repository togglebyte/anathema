use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::{PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Layout};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, LocalPos, Nodes, Widget, WidgetFactory, WidgetStyle,
};

use crate::layout::single::Single;

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
}

impl Widget for Expand {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, _node_id: &NodeId) {
        self.axis.resolve(context, None);
        self.fill.resolve(context, None);
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
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

    use crate::testing::test_widget;

    #[test]
    fn expand_border() {
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
                    "expand",
                    None,
                    [("factor".to_string(), 1.into())],
                    [expression(
                        "border",
                        None,
                        [],
                        [expression("expand", None, [], [])],
                    )],
                ),
                expression(
                    "expand",
                    None,
                    [("factor".to_string(), 2.into())],
                    [expression(
                        "border",
                        None,
                        [],
                        [expression("expand", None, [], [])],
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
                    "expand",
                    None,
                    [("factor".to_string(), 1.into())],
                    [expression(
                        "border",
                        None,
                        [],
                        [expression("expand", None, [], [])],
                    )],
                ),
                expression(
                    "expand",
                    None,
                    [("factor".to_string(), 2.into())],
                    [expression(
                        "border",
                        None,
                        [],
                        [expression("expand", None, [], [])],
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

    #[test]
    fn expand_horz() {
        let border = expression(
            "border",
            None,
            [],
            [expression(
                "expand",
                None,
                [("axis".to_string(), "horz".into())],
                [expression(
                    "text",
                    Some("A cup of tea please".into()),
                    [],
                    [],
                )],
            )],
        );

        test_widget(
            border,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════════════════╗
            ║┌────────────────────────────┐║
            ║│A cup of tea please         │║
            ║└────────────────────────────┘║
            ║                              ║
            ║                              ║
            ╚══════════════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expand_vert() {
        let border = expression(
            "border",
            None,
            [],
            [expression(
                "expand",
                None,
                [("axis".to_string(), "vert".into())],
                [expression(
                    "text",
                    Some("A cup of tea please".into()),
                    [],
                    [],
                )],
            )],
        );

        test_widget(
            border,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════════════════╗
            ║┌───────────────────┐         ║
            ║│A cup of tea please│         ║
            ║│                   │         ║
            ║│                   │         ║
            ║│                   │         ║
            ║│                   │         ║
            ║└───────────────────┘         ║
            ╚══════════════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expand_all() {
        let border = expression(
            "border",
            None,
            [],
            [expression(
                "expand",
                None,
                [],
                [expression(
                    "text",
                    Some("A cup of tea please".into()),
                    [],
                    [],
                )],
            )],
        );

        test_widget(
            border,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════════════════╗
            ║┌────────────────────────────┐║
            ║│A cup of tea please         │║
            ║│                            │║
            ║│                            │║
            ║│                            │║
            ║│                            │║
            ║└────────────────────────────┘║
            ╚══════════════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expand_with_padding() {
        let border = expression(
            "border",
            None,
            [("padding".to_string(), 1.into())],
            [expression(
                "expand",
                None,
                [],
                [expression(
                    "text",
                    Some("A cup of tea please".into()),
                    [],
                    [],
                )],
            )],
        );

        test_widget(
            border,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════════════════╗
            ║┌────────────────────────────┐║
            ║│                            │║
            ║│ A cup of tea please        │║
            ║│                            │║
            ║│                            │║
            ║│                            │║
            ║│                            │║
            ║└────────────────────────────┘║
            ╚══════════════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn expanding_inside_vstack() {
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
                        "hstack",
                        None,
                        [],
                        [
                            expression("text", Some("A cup of tea please".into()), [], []),
                            expression("spacer", None, [], []),
                        ],
                    )],
                ),
                expression(
                    "expand",
                    None,
                    [],
                    [expression(
                        "border",
                        None,
                        [],
                        [expression(
                            "expand",
                            None,
                            [],
                            [expression("text", Some("Hello world".into()), [], [])],
                        )],
                    )],
                ),
            ],
        );

        test_widget(
            vstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════════════════╗
            ║┌────────────────────────────┐║
            ║│A cup of tea please         │║
            ║└────────────────────────────┘║
            ║┌────────────────────────────┐║
            ║│Hello world                 │║
            ║│                            │║
            ║│                            │║
            ║└────────────────────────────┘║
            ╚══════════════════════════════╝
            "#,
            ),
        );
    }
}
