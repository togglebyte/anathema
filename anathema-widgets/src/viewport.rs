use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::{PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::{AnyWidget, FactoryContext, LayoutNodes, Nodes, Widget, WidgetFactory};

use crate::layout::many::Many;

/// A viewport where the children can be rendered with an offset.
#[derive(Debug)]
pub struct Viewport {
    /// Line / cell offset
    pub offset: Value<i32>,
    /// Clamp the horizontal / vertical space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp: Value<bool>,
    /// Layout direction.
    /// `Direction::Forward` is the default, and keeps the scroll position on the first child.
    /// `Direction::Backward` keeps the scroll position on the last child.
    pub direction: Value<Direction>,
    /// Vertical or horizontal
    pub axis: Value<Axis>,
}

impl Viewport {
    pub fn offset(&self) -> i32 {
        let mut offset = self.offset.value_or_default();

        if self.clamp.value_or(false) && offset < 0 {
            offset = 0;
        }

        offset
    }
}

impl Widget for Viewport {
    fn kind(&self) -> &'static str {
        "Viewport"
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        let mut many = Many::new(
            self.direction.value_or_default(),
            self.axis.value_or(Axis::Vertical),
            self.offset(),
            true,
        );

        many.layout(nodes)
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.direction.resolve(context, node_id);
        self.axis.resolve(context, node_id);
        self.offset.resolve(context, node_id);
        self.clamp.resolve(context, node_id);
    }

    fn position<'tpl>(&mut self, children: &mut Nodes<'_>, ctx: PositionCtx) {
        let direction = self.direction.value_or_default();
        let axis = self.axis.value_or(Axis::Vertical);
        let mut pos = ctx.pos;
        let mut offset = self.offset();

        // If the value is clamped, update the offset
        if self.clamp.value_or_default() {
            match axis {
                Axis::Horizontal => {
                    let total = children
                        .iter_mut()
                        .map(|(w, _)| w.size.width)
                        .sum::<usize>();

                    let h = ctx.inner_size.width as i32 + offset;
                    if h > total as i32 {
                        offset -= h - total as i32;
                    }
                }
                Axis::Vertical => {
                    let total = children
                        .iter_mut()
                        .map(|(w, _)| w.size.height)
                        .sum::<usize>();

                    let v = ctx.inner_size.height as i32 + offset;
                    if v > total as i32 {
                        offset -= v - total as i32;
                    }
                }
            };
        }

        if let Direction::Backwards = direction {
            match axis {
                Axis::Horizontal => pos.x += ctx.inner_size.width as i32,
                Axis::Vertical => pos.y += ctx.inner_size.height as i32,
            }
        }

        let offset = match direction {
            Direction::Forwards => -offset,
            Direction::Backwards => offset,
        };

        match axis {
            Axis::Horizontal => pos.x += offset,
            Axis::Vertical => pos.y += offset,
        }

        for (widget, children) in children.iter_mut() {
            if let Direction::Forwards = direction {
                widget.position(children, pos);
            }

            match direction {
                Direction::Forwards => match axis {
                    Axis::Horizontal => pos.x += widget.size.width as i32,
                    Axis::Vertical => pos.y += widget.size.height as i32,
                },
                Direction::Backwards => match axis {
                    Axis::Horizontal => pos.x -= widget.size.width as i32,
                    Axis::Vertical => pos.y -= widget.size.height as i32,
                },
            }

            if let Direction::Backwards = direction {
                widget.position(children, pos);
            }
        }
    }

    fn paint(&mut self, children: &mut Nodes<'_>, mut ctx: PaintCtx<'_, WithSize>) {
        let region = ctx.create_region();
        for (widget, children) in children.iter_mut() {
            let mut ctx = ctx.to_unsized();
            ctx.set_region(&region);
            widget.paint(children, ctx);
        }
    }
}

pub(crate) struct ViewportFactory;

impl WidgetFactory for ViewportFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = Viewport {
            direction: ctx.get("direction"),
            axis: ctx.get("axis"),
            offset: ctx.get("offset"),
            clamp: ctx.get("clamp"),
        };

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::expressions::Expression;
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    fn children(count: usize) -> Vec<Expression> {
        (0..count)
            .map(|i| {
                expression(
                    "border",
                    None,
                    [],
                    [expression("text", Some(i.into()), [], [])],
                )
            })
            .collect()
    }

    #[test]
    fn vertical_viewport() {
        let viewport = expression("viewport", None, [], children(10));
        test_widget(
            viewport,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn horizontal_viewport() {
        let viewport = expression(
            "viewport",
            None,
            [("axis".into(), "horz".into())],
            children(10),
        );
        test_widget(
            viewport,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐┌─┐┌─┐┌─┐┌─┐║
            ║│0││1││2││3││4│║
            ║└─┘└─┘└─┘└─┘└─┘║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn viewport_clamped() {
        let viewport = expression(
            "viewport",
            None,
            [
                ("clamp".into(), true.into()),
                ("offset".into(), (-2).into()),
            ],
            children(10),
        );
        test_widget(
            viewport,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    // #[test]
    // fn vertical_viewport_reversed() {
    //     let viewport = expression("viewport", None, [("direction".into(), "backward".into())], children(10));
    //     test_widget(
    //         viewport,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [═╗
    //         ║┌─┐            ║
    //         ║│8│            ║
    //         ║└─┘            ║
    //         ║┌─┐            ║
    //         ║│9│            ║
    //         ║└─┘            ║
    //         ╚═══════════════╝
    //         "#,
    //         ),
    //     );
    // }

    // #[test]
    // fn horizontal_viewport_reversed() {
    //     let body = children(10);
    //     test_widget(
    //         Viewport::new(Direction::Backward, Axis::Horizontal, 0),
    //         body,
    //         FakeTerm::from_str(
    //             r#"
    //         ╔═] Fake term [═╗
    //         ║┌─┐┌─┐┌─┐┌─┐┌─┐║
    //         ║│5││6││7││8││9│║
    //         ║└─┘└─┘└─┘└─┘└─┘║
    //         ║               ║
    //         ║               ║
    //         ║               ║
    //         ╚═══════════════╝
    //         "#,
    //         ),
    //     );
    // }

    #[test]
    fn vertical_forward_offset() {
        let viewport = expression(
            "viewport",
            None,
            [("offset".into(), 2.into())],
            children(10),
        );
        test_widget(
            viewport,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│2│            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn horizontal_forward_offset() {
        let viewport = expression(
            "viewport",
            None,
            [("axis".into(), "horz".into()), ("offset".into(), 2.into())],
            children(10),
        );
        test_widget(
            viewport,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┐┌─┐┌─┐┌─┐┌─┐┌─║
            ║││1││2││3││4││5║
            ║┘└─┘└─┘└─┘└─┘└─║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
