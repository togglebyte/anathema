use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::PositionCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{AnyWidget, FactoryContext, LayoutNodes, Nodes, Widget, WidgetFactory};

use crate::layout::stacked::Stacked;

/// Unlike the [`HStack`](crate::HStack) or the [`VStack`](crate::VStack) the [`ZStack`] draws the
/// children on top of each other.
///
/// This makes it possible to draw widgets on top of other widgets.
///
/// An example adding a title to a border
/// ```ignore
/// use anathema_widgets::{ZStack, Position, Border, Text, Widget, NodeId, HorzEdge, VertEdge};
///
/// let mut zstack = ZStack::new(None, None).into_container(NodeId::anon());
///
/// // Border
/// let mut border = Border::thin(20, 5).into_container(NodeId::anon());
/// border.add_child(Text::with_text("Here is some text").into_container(NodeId::anon()));
/// zstack.add_child(border);
///
/// // Title
/// let mut position = Position::new(HorzEdge::Left(1), VertEdge::Top(0)).into_container(NodeId::anon());
/// position.add_child(Text::with_text("] Title [").into_container(NodeId::anon()));
/// zstack.add_child(position);
/// ```
/// output
/// ```text
/// ┌] Title [─────────┐
/// │Here is some text │
/// │                  │
/// │                  │
/// └──────────────────┘
/// ```
///
/// Note that widgets are drawn in the order they are inserted.
/// To make something like a dialogue box appear on top it would have to be the last child of the
/// `ZStack`.
#[derive(Debug)]
pub struct ZStack {
    /// Width
    pub width: Value<usize>,
    /// Height
    pub height: Value<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Value<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Value<usize>,
}

impl ZStack {
    /// Create a new instance of a `ZStack`
    pub fn new(width: Value<usize>, height: Value<usize>) -> Self {
        Self {
            width,
            height,
            min_width: Value::Empty,
            min_height: Value::Empty,
        }
    }
}

impl Widget for ZStack {
    fn kind(&self) -> &'static str {
        "ZStack"
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.width.resolve(context, node_id);
        self.min_width.resolve(context, node_id);
        self.height.resolve(context, node_id);
        self.min_height.resolve(context, node_id);
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        if let Some(min_width) = self.min_width.value() {
            nodes.constraints.min_width = nodes.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height.value() {
            nodes.constraints.min_height = nodes.constraints.min_height.max(min_height);
        }
        if let Some(width) = self.width.value() {
            nodes.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height.value() {
            nodes.constraints.make_height_tight(height);
        }

        Stacked.layout(nodes)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes<'_>, ctx: PositionCtx) {
        for (widget, children) in children.iter_mut() {
            widget.position(children, ctx.pos);
        }
    }
}

pub(crate) struct ZStackFactory;

impl WidgetFactory for ZStackFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let mut widget = ZStack::new(context.get("width"), context.get("height"));
        widget.min_width = context.get("min-width");
        widget.min_height = context.get("min-height");
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    #[test]
    fn border_title() {
        let zstack = expression(
            "zstack",
            None,
            [],
            [
                expression("border", None, [], [expression("expand", None, [], [])]),
                expression(
                    "position",
                    None,
                    [("left".to_string(), 2.into())],
                    [expression("text", Some(" [title] ".into()), [], [])],
                ),
            ],
        );

        test_widget(
            zstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║┌─ [title] ────────┐║
            ║│                  │║
            ║│                  │║
            ║│                  │║
            ║└──────────────────┘║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn place_on_top() {
        let zstack = expression(
            "zstack",
            None,
            [],
            [
                expression("text", Some("000".into()), [], []),
                expression("text", Some("11".into()), [], []),
                expression("text", Some("2".into()), [], []),
            ],
        );

        test_widget(
            zstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║210                 ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }
}
