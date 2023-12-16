use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::PositionCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::{
    AnyWidget, Axis, FactoryContext, LayoutNodes, Nodes, Widget, WidgetFactory,
};

use crate::stack::Stack;

/// A widget that lays out its children horizontally.
/// ```text
/// ┌─┐┌─┐┌─┐┌─┐
/// │1││2││3││4│
/// └─┘└─┘└─┘└─┘
/// ```
///
/// ```ignore
/// use anathema_widgets::{HStack, Text, Widget, NodeId};
/// let mut hstack = HStack::new(None, None);
/// hstack.children.push(Text::with_text("1").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("2").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("3").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("4").into_container(NodeId::anon()));
/// ```
/// output:
/// ```text
/// 1234
/// ```
#[derive(Debug)]
pub struct HStack(Stack);

impl HStack {
    /// Create a new instance of an `HStack`.
    pub fn new(width: Value<usize>, height: Value<usize>) -> Self {
        Self(Stack::new(width, height, Axis::Horizontal))
    }
}

impl Widget for HStack {
    fn kind(&self) -> &'static str {
        "HStack"
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.0.update(context, node_id)
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        self.0.layout(nodes)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes<'_>, ctx: PositionCtx) {
        self.0.position(children, ctx)
    }
}

pub(crate) struct HStackFactory;

impl WidgetFactory for HStackFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let width = context.get("width");
        let height = context.get("height");
        let mut widget = HStack::new(width, height);
        widget.0.min_width = context.get("min-width");
        widget.0.min_height = context.get("min-height");
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
    fn only_hstack() {
        let hstack = expression("hstack", None, [], children(3));

        let _body = children(3);
        test_widget(
            hstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐┌─┐┌─┐      ║
            ║│0││1││2│      ║
            ║└─┘└─┘└─┘      ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn fixed_width_stack() {
        let hstack = expression(
            "hstack",
            None,
            [("width".to_string(), 6.into())],
            children(10),
        );
        test_widget(
            hstack,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐┌─┐         ║
            ║│0││1│         ║
            ║└─┘└─┘         ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
