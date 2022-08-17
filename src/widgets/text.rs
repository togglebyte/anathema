use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::antstring::AntString;
use crate::display::{Size, Style};
use crate::widgets::layout::text::TextLayout;

use super::{
    fields, LayoutCtx, LocalPos, NodeId, PaintCtx, PositionCtx, UpdateCtx, Widget, WidgetContainer, WithSize, Wrap,
};

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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
#[derive(Debug)]
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
    pub spans: Vec<WidgetContainer>,

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
    pub fn with_text(text: impl Into<String>) -> Self {
        let mut inst = Self::new();
        inst.set_text(text);
        inst
    }

    /// Create an instance of an unstyled `TextSpan` and add that as a child widget
    pub fn add_span(&mut self, text: impl Into<String>) {
        let span: TextSpan = text.into().into();
        self.spans.push(span.into_container(NodeId::auto()));
    }

    /// Update the text of the first `TextSpan` with new text, without changing attributes or `NodeId`.
    /// If there are no `TextSpan` one will be created and inserted.
    pub fn set_text(&mut self, text: impl Into<String>) {
        match self.spans.first_mut() {
            Some(first) => first.to::<TextSpan>().text = text.into(),
            None => self.spans.push(TextSpan::new(text).into_container(NodeId::auto())),
        }

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
        self.spans.get_mut(index).map(|span| &mut span.to::<TextSpan>().text)
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

        let string_slices = self
            .spans
            .iter_mut()
            .map(WidgetContainer::to::<TextSpan>)
            .map(|span| (&span.style, span.text.as_str()))
            .collect::<Vec<(&Style, &str)>>();
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
        let texts = self
            .spans
            .iter_mut()
            .map(WidgetContainer::to::<TextSpan>)
            .map(|span| (&span.style, span.text.as_str()))
            .collect::<Vec<(&Style, &str)>>();

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
        self.spans.iter_mut().collect()
    }

    fn add_child(&mut self, span: WidgetContainer) {
        if span.kind() == TextSpan::KIND {
            self.spans.push(span);
        }
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(pos) = self.spans.iter().position(|c| c.id.eq(child_id)) {
            return Some(self.spans.remove(pos));
        }

        None
    }

    fn update(&mut self, ctx: UpdateCtx) {
        if ctx.attributes.is_empty() {
            return;
        }

        if let Some(span) = self.spans.first_mut() {
            let span = span.to::<TextSpan>();
            ctx.attributes.update_style(&mut span.style);
        }

        for (k, _) in &ctx.attributes {
            match k.as_str() {
                fields::TRIM_START => self.trim_start = ctx.attributes.trim_start(),
                fields::TRIM_END => self.trim_end = ctx.attributes.trim_end(),
                fields::COLLAPSE_SPACES => self.collapse_spaces = ctx.attributes.collapse_spaces(),
                fields::WRAP => self.word_wrap = ctx.attributes.word_wrap(),
                fields::TEXT_ALIGN => self.text_alignment = ctx.attributes.text_alignment(),
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
    const KIND: &'static str = "TextSpan";

    /// Create a new instance of a text span
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), style: Style::new() }
    }
}

impl<S: Into<String>> From<S> for TextSpan {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

impl Widget for TextSpan {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, _: LayoutCtx) -> Size {
        Size::ZERO
    }

    fn position(&mut self, _: PositionCtx) {}

    fn paint(&mut self, _: PaintCtx<'_, WithSize>) {}

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        vec![]
    }

    fn add_child(&mut self, _: WidgetContainer) {}

    fn remove_child(&mut self, _: &NodeId) -> Option<WidgetContainer> {
        None
    }

    fn update(&mut self, ctx: UpdateCtx) {
        ctx.attributes.update_style(&mut self.style);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::display::Screen;
    use crate::widgets::testing::test_widget;
    use crate::widgets::Constraints;
    use crate::widgets::{Align, Alignment, Attributes, Border, BorderStyle, Padding, Pos, Sides};

    fn test_text(text: impl Widget, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        border.child = Some(text.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn qwerty_party_word_wrap() {
        let constraint = Constraints::new(8, None);
        let text = "hello how are you";
        let mut text_widget = Text::with_text(text);
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
        text.add_span("two");
        text.add_span(" three");
        text.add_span(" four ");
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
        let mut text_widget = Text::with_text(text);
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
        assert!(text.needs_layout());
        assert!(text.needs_paint());

        text.layout(LayoutCtx::new(Constraints::unbounded(), false, Padding::ZERO));
        text.paint(PaintCtx::new(&mut screen, None).into_sized(Size::ZERO, Pos::ZERO));
        assert!(!text.needs_layout());
        assert!(!text.needs_paint());

        text.add_span("change");
        assert!(text.needs_layout());
        assert!(text.needs_paint());
    }

    #[test]
    fn style_changes_via_attributes() {
        let mut text = Text::with_text("first span").into_container(NodeId::auto());
        text.update(Attributes::new("italic", true));
        let span = &text.to::<Text>().spans[0].to::<TextSpan>();
        assert!(span.style.attributes.contains(crate::display::Attributes::ITALIC));
    }
}
