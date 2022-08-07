use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::antstring::AntString;
use crate::display::{Size, Style};
use crate::widgets::layout::text::TextLayout;
use crate::widgets::{fields, Attributes, Wrap};

use super::LocalPos;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// Text alignment aligns the text inside its parent.
///
/// Given a border with a width of nine and text alignment set to [`TextAlignment::Right`]:
/// ```text
/// ┌───────┐
/// │I would│
/// │ like a│
/// │ lovely│
/// │ cup of│
/// │    tea│
/// │ please│
/// └───────┘
/// ```
///
/// The text will only align it self within the parent widget.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TextAlignment {
    /// Align the to the left inside the parent
    Left,
    /// Align the text in the centre of the parent
    Centre,
    /// Align the to the right inside the parent
    Right,
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
#[derive(Debug, Clone)]
pub struct Text {
    /// Trim any white space from the start of the string. Note that if `collapse_spaces` is `true`
    /// then all but one space will be trimmed from the start of the text.
    pub trim_start: bool,
    /// Trim any white space from the end of the string. Note that if `collapse_spaces` is `true`
    /// then all but one space will be trimmed from the end of the text.
    pub trim_end: bool,
    /// Remove any excess spaces if more than one is present.
    /// If both `trim_start` and `trim_end` are `false`, and `collapse_spaces` is true, this would convert
    /// ```text
    /// "   abc   "
    /// ```
    /// into
    /// ```text
    /// " abc "
    /// ```
    pub collapse_spaces: bool,
    /// Word wrapping
    pub word_wrap: Wrap,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: TextAlignment,
    /// Represents chunks of text that can have its own style. This makes it possible to style
    /// different parts of the same text.
    pub spans: Vec<TextSpan>,

    max_width: usize,
    previous_width: usize,
    previous_height: usize,
    needs_layout: bool,
    needs_paint: bool,
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    /// Create a new instance of a `Text` widget
    pub fn new() -> Self {
        Self {
            spans: vec![],
            max_width: 0,
            previous_width: 0,
            previous_height: 0,
            needs_layout: true,
            needs_paint: true,
            word_wrap: Wrap::Word,
            text_alignment: TextAlignment::Left,
            trim_start: true,
            trim_end: true,
            collapse_spaces: true,
        }
    }

    /// Remove all spans (all text)
    pub fn clear(&mut self) {
        self.spans.clear();
        self.needs_layout = true;
        self.needs_paint = true;
    }

    /// Create an instance of a `Text` widget with some inital unstyled text
    pub fn with_text(text: impl AsRef<str>) -> Self {
        let mut inst = Self::new();
        inst.set_text(text);
        inst
    }

    /// Replace the first text `Span` with some new text.
    /// If there are no `Span`s one will be inserted
    pub fn set_text(&mut self, text: impl AsRef<str>) {
        if !self.spans.is_empty() {
            let mut span = self.spans.remove(0);
            self.spans.clear();
            span.text = text.as_ref().to_string();
            self.spans.push(span);
        } else {
            let span = TextSpan::new(text);
            self.spans.push(span);
        }

        self.needs_layout = true;
        self.needs_paint = true;
    }

    /// When diffing two text widgets, only update the spans that have changed.
    /// There should be no need to use this outside of the templates project.
    pub fn update_span(&mut self, index: usize, text: Option<String>, style: Option<Style>) {
        if text.is_none() && style.is_none() {
            return;
        }

        let span = &mut self.spans[index];

        if let Some(text) = text {
            span.text = text;
        }

        if let Some(style) = style {
            span.style = style;
        }

        self.needs_layout = true;
        self.needs_paint = true;
    }

    /// Add another text span
    pub fn add_span(&mut self, span: impl Into<TextSpan>) {
        self.spans.push(span.into());
        self.needs_layout = true;
        self.needs_paint = true;
    }

    /// Get a mutable reference to the inner span's String
    /// ```
    /// # use anathema::widgets::{Text, WidgetContainer};
    /// # fn run(root: &mut WidgetContainer) -> Option<()> {
    ///     let text = root.by_id("the-text")?.to::<Text>().get_text_mut(0)?;
    ///     text.push_str("updated");
    /// #   Some(())
    /// # }
    /// ```
    pub fn get_text_mut(&mut self, index: usize) -> Option<&mut String> {
        self.spans.get_mut(index).map(|span| &mut span.text)
    }

    /// Get a reference to the inner span's String
    /// ```
    /// # use anathema::widgets::{Text, WidgetContainer};
    /// # fn run(root: &mut WidgetContainer) -> Option<()> {
    ///     let text = root.by_id("the-text")?.to::<Text>().get_text(0)?;
    /// #   Some(())
    /// # }
    /// ```
    pub fn get_text(&self, index: usize) -> Option<&String> {
        self.spans.get(index).map(|span| &span.text)
    }
}

impl<T: Into<String>> From<T> for Text {
    fn from(text: T) -> Self {
        let mut s = Self::new();
        s.add_span(text.into());
        s
    }
}

impl Widget for Text {
    fn kind(&self) -> &'static str {
        "Text"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // Set `needs_layout` to true as soon as the text change.
    // It makes no sense to check the "shape" of the text as that requires an
    // actual layout to happen, making this redundant
    fn needs_layout(&mut self) -> bool {
        true
        // self.needs_layout
    }

    fn needs_paint(&self) -> bool {
        true
        // self.needs_paint
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        self.needs_layout = false;
        self.max_width = ctx.constraints.max_width;

        let string_slices =
            self.spans.iter().map(|span| (&span.style, span.text.as_str())).collect::<Vec<(&Style, &str)>>();
        let string = AntString::with_annotations(&string_slices);

        self.previous_width = string.width();
        self.previous_height = string_slices.len();

        if string.is_empty() {
            return Size::ZERO;
        }

        let mut text_layout = TextLayout::new(self.word_wrap, self.max_width);
        text_layout.trim_start = self.trim_start;
        text_layout.trim_end = self.trim_end;
        text_layout.collapse_spaces = self.collapse_spaces;
        let strings = text_layout.layout(string);

        let height = strings.len().min(ctx.constraints.max_height);
        let width = strings.iter().map(|s| s.width()).max().unwrap_or(0).min(ctx.constraints.max_width);

        Size { width, height }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        self.needs_paint = false;
        let texts = self.spans.iter().map(|span| (&span.style, span.text.as_str())).collect::<Vec<(&Style, &str)>>();

        if texts.is_empty() {
            return;
        }

        if ctx.local_size.width == 0 {
            return;
        }

        let mut text_layout = TextLayout::new(self.word_wrap, self.max_width);
        text_layout.trim_start = self.trim_start;
        text_layout.trim_end = self.trim_end;
        text_layout.collapse_spaces = self.collapse_spaces;

        let anotated_string = AntString::with_annotations(&texts);
        let strings = text_layout.layout(anotated_string);

        for (y, string) in strings.into_iter().enumerate() {
            let x = match self.text_alignment {
                TextAlignment::Left => 0,
                TextAlignment::Centre => ctx.local_size.width / 2 - string.width() / 2,
                TextAlignment::Right => ctx.local_size.width - string.width(),
            };

            let mut pos = LocalPos::new(x, y);
            for (style, c) in string.annotated_chars() {
                ctx.put(c, *style, pos);
                pos.x += c.width().unwrap_or(0);
            }
        }
    }

    fn position(&mut self, _ctx: PositionCtx) {}

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        vec![]
    }

    fn add_child(&mut self, _: WidgetContainer) {}

    fn remove_child(&mut self, _: &NodeId) -> Option<WidgetContainer> {
        None
    }

    fn update(&mut self, attributes: Attributes) {
        if attributes.is_empty() {
            return;
        }

        if let Some(span) = self.spans.first_mut() {
            attributes.update_style(&mut span.style);
        }

        for (k, _) in &attributes {
            match k.as_str() {
                fields::TRIM_START => self.trim_start = attributes.trim_start(),
                fields::TRIM_END => self.trim_end = attributes.trim_end(),
                fields::COLLAPSE_SPACES => self.collapse_spaces = attributes.collapse_spaces(),
                fields::WRAP => self.word_wrap = attributes.word_wrap(),
                fields::TEXT_ALIGN => self.text_alignment = attributes.text_alignment(),
                _ => {}
            }
        }

        self.needs_layout = true;
        self.needs_paint = true;
    }
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
    /// Create a new instance of a text span
    pub fn new(text: impl AsRef<str>) -> Self {
        Self { text: text.as_ref().to_owned(), style: Style::new() }
    }
}

impl<S: AsRef<str>> From<S> for TextSpan {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::display::Screen;
    use crate::widgets::testing::test_widget;
    use crate::widgets::Constraints;
    use crate::widgets::{Align, Alignment, Border, BorderStyle, Padding, Pos, Sides};

    fn test_text(text: impl Widget, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        border.child = Some(text.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn qwerty_party_word_wrap() {
        let constraint = Constraints::new(8, None);
        let text = "hello how are you";
        let mut text_widget = Text::default();
        text_widget.add_span(TextSpan::new(text));
        let actual = text_widget.layout(LayoutCtx::new(constraint, false, Padding::ZERO));
        let expected = Size::new(7, 3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn word_wrap_excessive_space() {
        // By default the text layout will strip all
        // excessive space and trim the starting space
        test_text(
            Text::with_text("    hello     how    are    you     "),
            r#"
            ┌────────┐
            │hello   │
            │how are │
            │you     │
            └────────┘
            "#,
        );
    }

    #[test]
    fn word_wrap() {
        test_text(
            Text::with_text("hello how are you"),
            r#"
            ┌────────┐
            │hello   │
            │how are │
            │you     │
            └────────┘
            "#,
        );
    }

    #[test]
    fn no_word_wrap() {
        let mut text = Text::with_text("hello how are you");
        text.word_wrap = Wrap::NoWrap;
        test_text(
            text,
            r#"
            ┌───────┐
            │hello h│
            │       │
            │       │
            └───────┘
            "#,
        );
    }

    #[test]
    fn break_word_wrap() {
        let mut text = Text::with_text("hellohowareyou");
        text.word_wrap = Wrap::Break;
        test_text(
            text,
            r#"
            ┌───────┐
            │helloho│
            │wareyou│
            │       │
            └───────┘
            "#,
        );
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let mut text = Text::with_text("one");
        text.spans.push("two".into());
        text.spans.push(" three".into());
        text.spans.push(" four ".into());
        test_text(
            text,
            r#"
            ┌──────────┐
            │onetwo    │
            │three four│
            └──────────┘
            "#,
        );
    }

    #[test]
    fn right_alignment() {
        let mut alignment = Alignment::new(Align::TopRight);
        let mut text = Text::with_text("a one xxxxxxxx");
        text.text_alignment = TextAlignment::Right;
        alignment.child = Some(text.into_container(NodeId::auto()));
        test_text(
            alignment,
            r#"
            ┌────────┐
            │   a one│
            │xxxxxxxx│
            └────────┘
            "#,
        );
    }

    #[test]
    fn right_alignment_with_flair() {
        let mut alignment = Alignment::new(Align::TopRight);
        let mut text = Text::with_text("a\none");
        text.text_alignment = TextAlignment::Right;

        alignment.child = Some(text.into_container(NodeId::auto()));

        test_text(
            alignment,
            r#"
            ┌────────┐
            │       a│
            │     one│
            └────────┘
            "#,
        );
    }

    #[test]
    fn centre_alignment() {
        let mut alignment = Alignment::new(Align::Left);
        let mut text = Text::with_text("a one xxxxxxxxx");
        text.text_alignment = TextAlignment::Centre;
        alignment.child = Some(text.into_container(NodeId::auto()));
        test_text(
            alignment,
            r#"
            ┌─────────┐
            │  a one  │
            │xxxxxxxxx│
            └─────────┘
            "#,
        );
    }

    #[test]
    fn centre_alignment_with_flair() {
        let mut alignment = Alignment::new(Align::Centre);
        let mut text = Text::with_text("a\none");
        text.text_alignment = TextAlignment::Centre;
        alignment.child = Some(text.into_container(NodeId::auto()));
        test_text(
            alignment,
            r#"
            ┌───────┐
            │   a   │
            │  one  │
            └───────┘
            "#,
        );
    }

    #[test]
    fn word_wrap_no_space() {
        let constraint = Constraints::new(8, None);
        let text = "helloxhowareyou";
        let mut text_widget = Text::default();
        text_widget.spans = vec![TextSpan::new(text)];
        let actual = text_widget.layout(LayoutCtx::new(constraint, false, Padding::ZERO));
        let expected = Size::new(8, 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn char_wrap() {
        let constraint = Constraints::new(8, None);
        let text = "hello how are";
        let mut text_widget = Text::with_text(text);
        let actual = text_widget.layout(LayoutCtx::new(constraint, false, Padding::ZERO));
        let expected = Size::new(7, 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn char_wrap_layout_with_no_text() {
        let constraint = Constraints::new(5, None);
        let mut text_widget = Text::default();
        let actual = text_widget.layout(LayoutCtx::new(constraint, false, Padding::ZERO));
        let expected = Size::ZERO;
        assert_eq!(expected, actual);
    }

    #[test]
    #[ignore]
    fn needs_layout_and_paint() {
        let mut screen = Screen::new(&mut vec![], Size::ZERO).unwrap();

        let mut text = Text::with_text("hi");
        assert_eq!(text.needs_layout(), true);
        assert_eq!(text.needs_paint(), true);

        text.layout(LayoutCtx::new(Constraints::unbounded(), false, Padding::ZERO));
        text.paint(PaintCtx::new(&mut screen, None).into_sized(Size::ZERO, Pos::ZERO));
        assert_eq!(text.needs_layout(), false);
        assert_eq!(text.needs_paint(), false);

        text.add_span("change");
        assert_eq!(text.needs_layout(), true);
        assert_eq!(text.needs_paint(), true);
    }

    #[test]
    fn style_changes_via_attributes() {
        let mut text = Text::with_text("first span").into_container(NodeId::auto());
        text.update(Attributes::new("italic", true));
        let span = &text.to::<Text>().spans[0];
        assert!(span.style.attributes.contains(crate::display::Attributes::ITALIC));
    }
}
