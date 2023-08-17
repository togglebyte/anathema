use std::fmt::Write;

use anathema_generator::DataCtx;
use anathema_render::{Size, Style};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::{
    AnyWidget, BucketRef, Cached, LocalPos, Nodes, Value, Widget, WidgetContainer, WidgetFactory, X,
};
use unicode_width::UnicodeWidthStr;

use crate::layout::text::{Entry, Range, TextAlignment, TextLayout, Wrap};

pub struct CachedString {
    cache: Cached<Value>,
    string: String,
}

impl CachedString {
    fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl From<Cached<Value>> for CachedString {
    fn from(cache: Cached<Value>) -> Self {
        Self {
            string: cache.to_string(),
            cache,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Text -
// -----------------------------------------------------------------------------
/// Text widget
/// ```ignore
/// Attributes:
/// * background
/// * foreground
/// * text-align
/// * trimstart
/// * trimend
/// * collapse_spaces <- rename this to something less stupid
/// * wordwrap
/// ```
///
/// Note: Spans, unlike other widgets, does not require a widget id
///
/// A `Text` widget will be as wide as its text.
pub struct Text {
    /// Word wrapping
    pub word_wrap: Cached<Wrap>,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: Cached<TextAlignment>,
    /// Text
    pub text: CachedString,
    /// Text style
    pub style: Style,

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
            (self.text.as_str(), self.style)
        } else {
            let span = &children[widget_index - 1].to_ref::<TextSpan>();
            (span.text.as_str(), span.style)
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
        match self
            .text_alignment
            .value_ref()
            .unwrap_or(&TextAlignment::Left)
        {
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

    fn layout<'widget, 'parent>(
        &mut self,
        children: &mut Nodes,
        mut ctx: LayoutCtx,
        data: &BucketRef<'_>,
    ) -> Result<Size> {
        self.layout = TextLayout::ZERO;
        let bucket = data.read();
        let max_size = Size::new(ctx.constraints.max_width, ctx.constraints.max_height);
        self.layout.set_max_size(max_size);
        self.layout
            .set_wrap(*self.word_wrap.value_ref().unwrap_or(&Wrap::Normal));
        self.layout.process(self.text.as_str());

        drop(bucket);

        while let Some((span, children)) = children.next(data).transpose()? {
            // Ignore any widget that isn't a span
            if span.kind() != TextSpan::KIND {
                continue;
            }

            let Some(inner_span) = span.try_to_mut::<TextSpan>() else {
                continue;
            };
            self.layout.process(inner_span.text.as_str());
        }

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
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    /// The text
    pub text: String,
    /// Style for the text
    pub style: Style,
}

impl TextSpan {
    const KIND: &'static str = "TextSpan";

    pub fn new() -> Self {
        Self {
            text: String::new(),
            style: Style::new(),
        }
    }
}

impl Widget for TextSpan {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, _: &mut Nodes, _: LayoutCtx, _: &BucketRef<'_>) -> Result<Size> {
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
    fn make(&self, data: DataCtx<WidgetContainer>) -> Result<Box<dyn AnyWidget>> {
        let word_wrap = data
            .get("wrap")
            .and_then(|scope_val| Cached::<Wrap>::new(scope_val, &data))
            .unwrap_or(Cached::Value(Wrap::Normal));
        let text_alignment = data
            .get("text-align")
            .and_then(|scope_val| Cached::new(scope_val, &data))
            .unwrap_or(Cached::Value(TextAlignment::Left));

        // TODO: we do need them styles
        // widget.style = values.style();

        // TODO: force the existence of a value
        let text = data
            .text
            .as_ref()
            .map(|s| s.to_scope_value::<X>(data.store, data.scope, data.node_id));

        let text = text
            .and_then(|scope_val| Cached::new(scope_val, &data))
            .expect("a text widget always has a text field");

        let mut widget = Text {
            word_wrap,
            text_alignment,
            style: Style::new(),
            layout: TextLayout::ZERO,
            text: text.into(),
        };

        Ok(Box::new(widget))
    }
}

pub(crate) struct SpanFactory;

impl WidgetFactory for SpanFactory {
    fn make(&self, data: DataCtx<WidgetContainer>) -> Result<Box<dyn AnyWidget>> {
        let mut widget = TextSpan::new();
        // if let Some(text) = text {
        //     widget.text = values.text_to_string(text).to_string();
        // }
        // widget.style = values.style();

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::template_span;
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn word_wrap_excessive_space() {
        test_widget(
            Text::new("hello      how are     you"),
            [],
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
            Text::new("hello how are you"),
            [],
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
        let mut text = Text::new("hello how are you");
        text.word_wrap = Wrap::Overflow;
        test_widget(
            text,
            [],
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
        let mut text = Text::new("hellohowareyoudoing");
        text.word_wrap = Wrap::WordBreak;
        test_widget(
            text,
            [],
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
            template_span("two"),
            template_span(" averylongword"),
            template_span(" bunny "),
        ];
        test_widget(
            Text::new("one"),
            body,
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
        let mut text = Text::new("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Right;
        test_widget(
            text,
            [],
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
        let mut text = Text::new("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Centre;
        test_widget(
            text,
            [],
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
