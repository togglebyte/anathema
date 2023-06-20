use anathema_render::{Size, Style};
use unicode_width::UnicodeWidthStr;

use super::{LocalPos, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize, Wrap};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::layout::text::{Entry, Range, TextLayout};
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{AnyWidget, TextAlignment, TextPath};

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
#[derive(Debug, PartialEq, Eq)]
pub struct Text {
    /// Word wrapping
    pub word_wrap: Wrap,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: TextAlignment,
    /// Text
    pub text: String,
    /// Text style
    pub style: Style,

    layout: TextLayout,
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    pub const KIND: &'static str = "Text";

    /// Create a new instance of a `Text` widget
    pub fn new() -> Self {
        Self {
            text: String::new(),
            style: Style::new(),
            word_wrap: Wrap::Normal,
            text_alignment: TextAlignment::Left,
            layout: TextLayout::ZERO,
        }
    }

    /// Create an instance of a `Text` widget with some inital unstyled text
    pub fn with_text(text: impl Into<String>) -> Self {
        let mut inst = Self::new();
        inst.set_text(text);
        inst
    }

    /// Update the text of the first `TextSpan` with new text, without changing attributes or `NodeId`.
    /// If there are no `TextSpan` one will be created and inserted.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    fn text_and_style<'gen: 'a, 'a>(
        &'a self,
        entry: &Entry,
        children: &'a [WidgetContainer<'gen>],
    ) -> (&'a str, Style) {
        let widget_index = match entry {
            Entry::All { index, .. } => index,
            Entry::Range(Range { slice, .. }) => slice,
            Entry::Newline => {
                unreachable!("when painting the lines, the NewLine is always skipped")
            }
        };

        let (text, style) = if *widget_index == 0 {
            (self.text.as_str(), self.style)
        } else {
            let span = children[widget_index - 1].to_ref::<TextSpan>();
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
        children: &[WidgetContainer<'_>],
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
        match self.text_alignment {
            TextAlignment::Left => {}
            TextAlignment::Centre => pos.x = max_width / 2 - line_width / 2,
            TextAlignment::Right => pos.x = max_width - line_width,
        }

        // ... then print the chars
        for entry_index in range.clone() {
            let entry = &self.layout.lines.inner[entry_index];
            let (text, style) = self.text_and_style(entry, children);
            let Some(new_pos) = ctx.print(text, style, pos) else { continue };
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

    fn layout<'tpl, 'parent>(&mut self, layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        let max_size = Size::new(layout.constraints.max_width, layout.constraints.max_height);
        self.layout.set_max_size(max_size);
        self.layout.set_wrap(self.word_wrap);

        self.layout.process(self.text.as_str());

        let mut values = layout.values.next();
        let mut gen = Generator::new(layout.templates, layout.lookup, &mut values);
        while let Some(mut span) = gen.next(&mut values).transpose()? {
            // Ignore any widget that isn't a span
            if span.kind() != TextSpan::KIND {
                continue;
            }

            let Some(inner_span) = span.try_to_mut::<TextSpan>() else { continue };
            self.layout.process(inner_span.text.as_str());
            layout.children.push(span);
        }

        Ok(self.layout.size())
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        let mut y = 0;
        let mut range = 0..0;
        for entry in &self.layout.lines.inner {
            match entry {
                Entry::All { .. } | Entry::Range(_) => range.end += 1,
                Entry::Newline => {
                    self.paint_line(&mut range, children, y, &mut ctx);
                    y += 1;
                    continue;
                }
            };
        }

        self.paint_line(&mut range, children, y, &mut ctx);
    }

    fn position<'gen, 'ctx>(&mut self, _: PositionCtx, _: &mut [WidgetContainer<'gen>]) {
        // NOTE: there is no need to position text as the text
        // is printed from the context position
    }

    // fn update(&mut self, ctx: UpdateCtx) {
    //     // if ctx.attributes.is_empty() {
    //     //     return;
    //     // }

    //     // if let Some(span) = self.spans.first_mut() {
    //     //     let span = span.to_mut::<TextSpan>();
    //     //     ctx.attributes.update_style(&mut span.style);
    //     // }

    //     // for (k, _) in &ctx.attributes {
    //     //     match k.as_str() {
    //     //         fields::TRIM_START => self.trim_start = ctx.attributes.trim_start(),
    //     //         fields::TRIM_END => self.trim_end = ctx.attributes.trim_end(),
    //     //         fields::COLLAPSE_SPACES => self.collapse_spaces = ctx.attributes.collapse_spaces(),
    //     //         fields::WRAP => self.word_wrap = ctx.attributes.word_wrap(),
    //     //         fields::TEXT_ALIGN => self.text_alignment = ctx.attributes.text_alignment(),
    //     //         _ => {}
    //     //     }
    //     // }

    //     // self.needs_layout = true;
    //     // self.needs_paint = true;
    // }
}

/// Represents a chunk of text with its own style
#[derive(Debug, Clone)]
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

    fn layout(&mut self, _: LayoutCtx<'_, '_, '_>) -> Result<Size> {
        unreachable!("layout should never be called directly on a span");
    }

    fn position<'gen, 'ctx>(&mut self, _: PositionCtx, _: &mut [WidgetContainer<'gen>]) {
        // NOTE: there is no need to position text as the text is printed
        // from the context position
    }

    fn paint<'gen, 'ctx>(&mut self, _: PaintCtx<'_, WithSize>, _: &mut [WidgetContainer<'gen>]) {
        panic!("don't invoke paint on the span directly.");
    }

    // fn update(&mut self, ctx: UpdateCtx) {
    //     ctx.attributes.update_style(&mut self.style);
    // }
}

pub(crate) struct TextFactory;

impl WidgetFactory for TextFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let mut widget = Text::new();
        widget.word_wrap = values.word_wrap();
        widget.text_alignment = values.text_alignment();
        widget.style = values.style();
        if let Some(text) = text {
            widget.text = values.text_to_string(text).to_string();
        }
        Ok(Box::new(widget))
    }
}

pub(crate) struct SpanFactory;

impl WidgetFactory for SpanFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let mut widget = TextSpan::new();
        if let Some(text) = text {
            widget.text = values.text_to_string(text).to_string();
        }
        widget.style = values.style();

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::template::{template_span, Template};
    use crate::testing::{test_widget, FakeTerm};
    use crate::{fields, Attributes, Border, BorderStyle, Lookup, Sides, TextPath};

    #[test]
    fn word_wrap_excessive_space() {
        test_widget(
            Text::with_text("hello      how are     you"),
            &[],
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
            Text::with_text("hello how are you"),
            &[],
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
        let mut text = Text::with_text("hello how are you");
        text.word_wrap = Wrap::Overflow;
        test_widget(
            text,
            &[],
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
        let mut text = Text::with_text("hellohowareyoudoing");
        text.word_wrap = Wrap::WordBreak;
        test_widget(
            text,
            &[],
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
            Text::with_text("one"),
            &body,
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
        let mut text = Text::with_text("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Right;
        test_widget(
            text,
            &[],
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
        let mut text = Text::with_text("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Centre;
        test_widget(
            text,
            &[],
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
