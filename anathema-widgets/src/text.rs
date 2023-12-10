use std::fmt::Write;

use anathema_render::{Size, Style};
use anathema_values::{Attributes, Context, DynValue, NodeId, Path, State, Value, ValueExpr};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::{
    AnyWidget, FactoryContext, LocalPos, Nodes, Widget, WidgetContainer, WidgetFactory, WidgetStyle, LayoutNodes,
};
use unicode_width::UnicodeWidthStr;

use crate::layout::text::{Entry, Range, TextAlignment, TextLayout, Wrap};

// -----------------------------------------------------------------------------
//     - Text -
// -----------------------------------------------------------------------------
/// Text widget
/// ```ignore
/// Attributes:
/// * background
/// * foreground
/// * text-align
/// * wordwrap
/// ```
///
/// Note: Spans, unlike other widgets, does not require a widget id
///
/// A `Text` widget will be as wide as its text.
#[derive(Debug)]
pub struct Text {
    /// Word wrapping
    pub word_wrap: Value<Wrap>,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: Value<TextAlignment>,
    /// Text
    pub text: Value<String>,
    /// Text style
    pub style: WidgetStyle,

    layout: TextLayout,
}

impl Text {
    pub const KIND: &'static str = "Text";

    fn text_and_style<'a>(
        &'a self,
        entry: &Entry,
        children: &[&'a WidgetContainer],
    ) -> (&'a str, Style) {
        let widget_index = match entry {
            Entry::All { index, .. } => index,
            Entry::Range(Range { slice, .. }) => slice,
            Entry::Newline => {
                unreachable!("when painting the lines, the `Entry::NewLine` is always skipped")
            }
        };

        let (text, style) = if *widget_index == 0 {
            (self.text.str(), self.style.style())
        } else {
            let span = &children[widget_index - 1].to_ref::<TextSpan>();
            (span.text.str(), span.style.style())
        };

        if let Entry::Range(Range { start, end, .. }) = entry {
            (&text[*start..*end], style)
        } else {
            (text, style)
        }
    }

    fn paint_line(
        &self,
        range: &mut std::ops::Range<usize>,
        children: &[&WidgetContainer],
        y: usize,
        ctx: &mut PaintCtx<'_, WithSize>,
    ) {
        let mut pos = LocalPos::new(0, y);

        // Calculate the line width.
        // This is only relevant if `Align` is not `Left`.
        let mut line_width = 0;
        for entry in &self.layout.lines.inner[range.clone()] {
            let (text, _) = self.text_and_style(entry, children);
            line_width += text.width();
        }

        let max_width = self.layout.size().width;
        match self.text_alignment.value_or_default() {
            TextAlignment::Left => {}
            TextAlignment::Centre => pos.x = max_width / 2 - line_width / 2,
            TextAlignment::Right => pos.x = max_width - line_width,
        }

        // ... then print the chars
        for entry_index in range.clone() {
            let entry = &self.layout.lines.inner[entry_index];
            let (text, style) = self.text_and_style(entry, children);
            let Some(new_pos) = ctx.print(text, style, pos) else {
                continue;
            };
            pos = new_pos;
        }

        range.end += 1;
        range.start = range.end;
    }
}

impl Widget for Text {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.word_wrap.resolve(context, None);
        self.text_alignment.resolve(context, None);
        self.text.resolve(context, None);
        self.style.resolve(context, None);
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        self.layout = TextLayout::ZERO;
        let max_size = Size::new(nodes.constraints.max_width, nodes.constraints.max_height);
        self.layout.set_max_size(max_size);

        self.word_wrap
            .value_ref()
            .map(|wrap| self.layout.set_wrap(*wrap));
        self.layout.process(self.text.str());

        nodes.for_each(|mut span| {
            // Ignore any widget that isn't a span
            if span.kind() != TextSpan::KIND {
                return Ok(());
            }

            let inner_span = span.to_mut::<TextSpan>();

            self.layout.process(inner_span.text.str());
            Ok(())
        });

        Ok(self.layout.size())
    }

    fn paint<'ctx>(&mut self, children: &mut Nodes, mut ctx: PaintCtx<'_, WithSize>) {
        let mut y = 0;
        let mut range = 0..0;
        let children = children.iter_mut().map(|(c, _)| &*c).collect::<Vec<_>>();
        for entry in &self.layout.lines.inner {
            match entry {
                Entry::All { .. } | Entry::Range(_) => range.end += 1,
                Entry::Newline => {
                    self.paint_line(&mut range, &children, y, &mut ctx);
                    y += 1;
                    continue;
                }
            };
        }

        self.paint_line(&mut range, &children, y, &mut ctx);
    }

    fn position<'ctx>(&mut self, _: &mut Nodes, _: PositionCtx) {
        // NOTE: there is no need to position text as the text
        // is printed from the context position
    }
}

/// Represents a chunk of text with its own style
#[derive(Debug)]
pub struct TextSpan {
    /// The text
    pub text: Value<String>,
    /// Style for the text
    pub style: WidgetStyle,
}

impl TextSpan {
    const KIND: &'static str = "TextSpan";
}

impl Widget for TextSpan {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.text.resolve(context, None);
        self.style.resolve(context, None);
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        panic!("layout should never be called directly on a span");
    }

    fn position<'ctx>(&mut self, _: &mut Nodes, _: PositionCtx) {
        // NOTE: there is no need to position text as the text is printed from the context position
        panic!("don't invoke position on the span directly.");
    }

    fn paint<'ctx>(&mut self, _: &mut Nodes, _: PaintCtx<'_, WithSize>) {
        panic!("don't invoke paint on the span directly.");
    }
}

pub(crate) struct TextFactory;

impl WidgetFactory for TextFactory {
    fn make(&self, mut ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let mut widget = Text {
            word_wrap: ctx.get("wrap"),
            text_alignment: ctx.get("text-align"),
            style: ctx.style(),
            layout: TextLayout::ZERO,
            text: ctx.text.take(),
        };
        widget.text.resolve(ctx.ctx, Some(&ctx.node_id));

        Ok(Box::new(widget))
    }
}

pub(crate) struct SpanFactory;

impl WidgetFactory for SpanFactory {
    fn make(&self, mut ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let mut widget = TextSpan {
            text: ctx.text.take(),
            style: ctx.style(),
        };
        widget.text.resolve(ctx.ctx, Some(&ctx.node_id));

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::testing::{expression, FakeTerm};

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn word_wrap_excessive_space() {
        test_widget(
            expression("text", Some("hello      how are     you"), [], []),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello      how  ║
            ║are     you     ║
            ║                ║
            ║                ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn word_wrap() {
        test_widget(
            expression("text", Some("hello how are you"), [], []),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are   ║
            ║you             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn no_word_wrap() {
        test_widget(
            expression(
                "text",
                Some("hello how are you"),
                [("wrap".into(), ValueExpr::from("overflow"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are yo║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn break_word_wrap() {
        test_widget(
            expression(
                "text",
                Some("hellohowareyoudoing"),
                [("wrap".into(), ValueExpr::from("break"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hellohowareyoudo║
            ║ing             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let body = [
            expression("span", Some("two"), [], []),
            expression("span", Some(" averylongword"), [], []),
            expression("span", Some(" bunny "), [], []),
        ];

        let text = expression("text", Some("one"), [], body);

        test_widget(
            text,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═════╗
            ║onetwo             ║
            ║averylongword bunny║
            ║                   ║
            ╚═══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn right_alignment() {
        test_widget(
            expression(
                "text",
                Some("a one xxxxxxxxxxxxxxxxxx"),
                [("text-align".into(), ValueExpr::from("right"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║             a one║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn centre_alignment() {
        test_widget(
            expression(
                "text",
                Some("a one xxxxxxxxxxxxxxxxxx"),
                [("text-align".into(), ValueExpr::from("center"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║       a one      ║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }
}
