use anathema_render::Size;

use super::{NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::layout::horizontal::Horizontal;
use crate::layout::Layouts;
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{AnyWidget, Axis, TextPath};

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
    pub width: Option<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
}

impl HStack {
    /// Create a new instance of an `HStack`.
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            min_width: None,
            min_height: None,
        }
    }
}

impl Widget for HStack {
    fn kind(&self) -> &'static str {
        "HStack"
    }

    fn layout(&mut self, mut ctx: LayoutCtx<'_, '_, '_>) -> Result<Size> {
        if let Some(width) = self.width {
            ctx.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height {
            ctx.constraints.make_height_tight(height);
        }
        if let Some(min_width) = self.min_width {
            ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        }

        Layouts::new(Horizontal, &mut ctx).layout()?.size()
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        let mut pos = ctx.pos;
        for widget in children {
            widget.position(pos);
            pos.x += widget.outer_size().width as i32;
        }
    }

    // fn update(&mut self, ctx: UpdateCtx) {
    //     if let Some(width) = ctx.attributes.width() {
    //         self.width = Some(width);
    //     }
    //     if let Some(height) = ctx.attributes.height() {
    //         self.height = Some(height);
    //     }
    // }
}

pub(crate) struct HStackFactory;

impl WidgetFactory for HStackFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let width = values.width();
        let height = values.height();
        let mut widget = HStack::new(width, height);
        widget.min_width = values.min_width();
        widget.min_height = values.min_height();
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::template::{template, template_text, Template};
    use crate::testing::{test_widget, FakeTerm};

    fn children(count: usize) -> Vec<Template> {
        (0..count)
            .map(|i| template("border", (), vec![template_text(i.to_string())]))
            .collect()
    }

    #[test]
    fn only_hstack() {
        let mut hstack = HStack::new(None, None);
        let body = children(3);
        test_widget(
            hstack,
            &body,
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
            )
        );
    }

    #[test]
    fn fixed_width_stack() {
        let mut hstack = HStack::new(6, None);
        let body = children(10);
        test_widget(
            hstack,
            &body,
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
            )
        );
    }
}
