use anathema_render::Size;
use anathema_values::{Attributes, Context, NodeId, Value};
use anathema_widget_core::contexts::{LayoutCtx, PositionCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Direction, Layout, Layouts};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, Nodes, Widget, WidgetContainer, WidgetFactory,
};

use crate::layout::horizontal::Horizontal;

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
pub struct HStack {
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Value<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Value<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Value<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Value<usize>,
}

impl HStack {
    /// Create a new instance of an `HStack`.
    pub fn new(width: Value<usize>, height: Value<usize>) -> Self {
        Self {
            width,
            height,
            min_width: Value::Empty,
            min_height: Value::Empty,
        }
    }
}

impl Widget for HStack {
    fn kind(&self) -> &'static str {
        "HStack"
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        if let Some(width) = self.width.value() {
            nodes.constraints.max_width = nodes.constraints.max_width.min(width);
        }
        if let Some(height) = self.height.value() {
            nodes.constraints.max_height = nodes.constraints.max_height.min(height);
        }
        if let Some(min_width) = self.min_width.value() {
            nodes.constraints.min_width = nodes.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height.value() {
            nodes.constraints.min_height = nodes.constraints.min_height.max(min_height);
        }

        Horizontal::new(Direction::Forward).layout(nodes)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        let mut pos = ctx.pos;
        for (widget, children) in children.iter_mut() {
            widget.position(children, pos);
            pos.x += widget.outer_size().width as i32;
        }
    }
}

pub(crate) struct HStackFactory;

impl WidgetFactory for HStackFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let width = context.get("width");
        let height = context.get("height");
        let mut widget = HStack::new(width, height);
        widget.min_width = context.get("min-width");
        widget.min_height = context.get("min-height");
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::{template, template_text, Template};
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    fn children(count: usize) -> Vec<Template> {
        (0..count)
            .map(|i| template("border", (), [template_text(i.to_string())]))
            .collect()
    }

    #[test]
    fn only_hstack() {
        let hstack = HStack::new(None, None);
        let body = children(3);
        test_widget(
            hstack,
            body,
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
        let hstack = HStack::new(6, None);
        let body = children(10);
        test_widget(
            hstack,
            body,
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
