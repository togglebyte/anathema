use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::PositionCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Direction, Layout};
use anathema_widget_core::{Axis, LayoutNodes, Nodes};

use crate::layout::horizontal::Horizontal;
use crate::layout::vertical::Vertical;

/// A widget that lays out its children vertically.
/// ```text
/// ┌─┐
/// │1│
/// └─┘
/// ┌─┐
/// │2│
/// └─┘
/// ┌─┐
/// │3│
/// └─┘
/// ```
///
/// ```ignore
/// use anathema_widgets::{VStack, Text, Widget, NodeId};
/// let mut vstack = VStack::new(None, None);
/// vstack.children.push(Text::with_text("1").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("2").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("3").into_container(NodeId::anon()));
/// ```
/// output:
/// ```text
/// 1
/// 2
/// 3
/// ```
#[derive(Debug)]
pub struct Stack {
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Value<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Value<usize>,
    /// The minimum width. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Value<usize>,
    /// The minimum height. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Value<usize>,
    axis: Axis,
}

impl Stack {
    /// Creates a new instance of a `VStack`
    pub fn new(width: Value<usize>, height: Value<usize>, axis: Axis) -> Self {
        Self {
            width,
            height,
            min_width: Value::Empty,
            min_height: Value::Empty,
            axis,
        }
    }
}

impl Stack {
    pub(crate) fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.width.resolve(context, node_id);
        self.min_width.resolve(context, node_id);
        self.height.resolve(context, node_id);
        self.min_height.resolve(context, node_id);
    }

    pub(crate) fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        if let Some(width) = self.width.value() {
            nodes.constraints.max_width = nodes.constraints.max_width.min(width);
            nodes.constraints.min_width = nodes.constraints.max_width.min(width);
        }

        if let Some(height) = self.height.value() {
            nodes.constraints.max_height = nodes.constraints.max_height.min(height);
            nodes.constraints.min_height = nodes.constraints.max_height.min(height);
        }

        if let Some(min_width) = self.min_width.value() {
            nodes.constraints.min_width = nodes.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height.value() {
            nodes.constraints.min_height = nodes.constraints.min_height.max(min_height);
        }

        match self.axis {
            Axis::Vertical => Vertical::new(Direction::Forward).layout(nodes),
            Axis::Horizontal => Horizontal::new(Direction::Forward).layout(nodes),
        }
    }

    pub(crate) fn position(&mut self, children: &mut Nodes<'_>, ctx: PositionCtx) {
        let mut pos = ctx.pos;
        for (widget, children) in children.iter_mut() {
            widget.position(children, pos);
            match self.axis {
                Axis::Vertical => pos.y += widget.outer_size().height as i32,
                Axis::Horizontal => pos.x += widget.outer_size().width as i32,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::expressions::Expression;
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    // TODO: there are many copies of this function...
    // just saying..
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
    fn only_vstack() {
        let vstack = expression("vstack", None, [], children(3));
        test_widget(
            vstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│2│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn fixed_height_stack() {
        let vstack = expression(
            "vstack",
            None,
            [("height".to_string(), 6.into())],
            children(10),
        );
        test_widget(
            vstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
