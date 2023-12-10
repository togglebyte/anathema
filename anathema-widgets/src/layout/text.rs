use anathema_render::Size;
use anathema_values::{impl_dyn_value, Resolver, ValueResolver, Context, DynValue, NodeId, Value, ValueExpr, ValueRef};
use unicode_width::UnicodeWidthChar;

fn is_word_boundary(c: char) -> bool {
    c == '-' || c.is_whitespace()
}

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
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum TextAlignment {
    /// Align the to the left inside the parent
    #[default]
    Left,
    /// Align the text in the centre of the parent
    Centre,
    /// Align the to the right inside the parent
    Right,
}

impl_dyn_value!(TextAlignment);

impl TryFrom<ValueRef<'_>> for TextAlignment {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        let wrap = match value {
            ValueRef::Str("center" | "centre") => Self::Centre,
            ValueRef::Str("right") => Self::Right,
            _ => Self::Left,
        };
        Ok(wrap)
    }
}

/// Word wrapping strategy
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Wrap {
    /// Normal word wrapping. This will break text on hyphen and whitespace.
    /// Trailing whitespace is consumed if it would cause a line break.
    Normal,
    /// Insert a newline in the middle of any text
    WordBreak,
    /// Don't wrap the text. If the text exceeds the maximum width it will be
    /// truncated
    Overflow,
}

impl_dyn_value!(Wrap);

impl TryFrom<ValueRef<'_>> for Wrap {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        let wrap = match value {
            ValueRef::Str("overflow") => Self::Overflow,
            ValueRef::Str("break") => Self::WordBreak,
            _ => Self::Normal,
        };
        Ok(wrap)
    }
}

/// A position is a specific byte index in a specific slice
#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct Pos {
    slice: usize,
    byte: usize,
}

/// A range inside a selected slice
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Range {
    /// Selected slice
    pub slice: usize,
    /// Starting byte
    pub start: usize,
    /// End byte
    pub end: usize,
}

/// A [`Line`] consists of multiple entries.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Entry {
    /// The entire slice fits inside a `Line`
    All { index: usize, len: usize },
    /// A part of a slice
    Range(Range),
    /// Insert a new line where this variant is found
    Newline,
}

/// Multiple lines, represented as a single vector.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Lines {
    pub(crate) inner: Vec<Entry>,
    current_slice: usize,
}

impl Lines {
    const fn new() -> Self {
        Self {
            inner: Vec::new(),
            current_slice: 0,
        }
    }

    fn insert(&mut self, entry_index: usize, new: Entry) {
        self.inner.insert(entry_index, new);
    }

    /// Remove the entry and return the total number of bytes for the entry.
    /// `Entry::Newline` should never be removed
    fn remove_get_len(&mut self, entry_index: usize) -> usize {
        match self.inner.remove(entry_index) {
            Entry::All { len, .. } => len,
            Entry::Range(Range { start, end, .. }) => end - start,
            Entry::Newline => unreachable!("new lines should not be removed"),
        }
    }

    /// Insert a range for the current slice
    fn range(&mut self, start: usize, end: usize) {
        let range = Range {
            slice: self.current_slice,
            start,
            end,
        };
        self.inner.push(Entry::Range(range));
    }

    /// Insert a Newline
    fn newline(&mut self) {
        self.inner.push(Entry::Newline);
    }

    /// Insert `Entry::All` for the current slice
    fn all(&mut self, slice_len: usize) {
        self.inner.push(Entry::All {
            index: self.current_slice,
            len: slice_len,
        });
    }
}

/// A word boundary
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct WordBoundary {
    // The width up to the word boundary
    width: usize,
    range: Range,
    entry_index: usize,
    skip: usize,
}

impl WordBoundary {
    fn new(width: usize, entry_index: usize, range: Range, skip: usize) -> Self {
        Self {
            width,
            range,
            entry_index,
            skip,
        }
    }
}

enum State {
    ExceedingCharWidth,
    Newline,
    MaxWidth,
    WordBoundary(WordBoundary, bool),
    Process,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextLayout {
    pub(crate) lines: Lines,
    pub(crate) max_size: Size,
    current_width: usize,
    word_boundary: Option<WordBoundary>,
    longest_line: usize,
    line_count: usize,
    wrap: Wrap,
}

impl TextLayout {
    pub const ZERO: Self = Self::new(Size::ZERO, Wrap::Normal);

    pub const fn new(max_size: Size, wrap: Wrap) -> Self {
        Self {
            lines: Lines::new(),
            max_size,
            current_width: 0,
            word_boundary: None,
            longest_line: 0,
            line_count: 0,
            wrap,
        }
    }

    fn check(&mut self) -> bool {
        if self.max_size.width == 0 {
            return false;
        }

        if self.max_size.height == 0 {
            return false;
        }

        true
    }

    pub fn set_max_size(&mut self, max_size: Size) {
        self.max_size = max_size;
    }

    pub fn set_wrap(&mut self, wrap: Wrap) {
        self.wrap = wrap;
    }

    pub fn size(&self) -> Size {
        let width = self.longest_line.max(self.current_width);
        if width == 0 {
            return Size::ZERO;
        }
        Size {
            width,
            height: self.line_count,
        }
    }

    fn state(&mut self, c: char, char_width: usize) -> State {
        if self.max_size.width == 1 && char_width > 1 {
            State::ExceedingCharWidth
        } else if c == '\n' {
            State::Newline
        } else if self.current_width + char_width > self.max_size.width {
            match self.word_boundary.take() {
                Some(_) if c.is_whitespace() => State::MaxWidth,
                Some(wb) => State::WordBoundary(wb, wb.entry_index == self.lines.inner.len()),
                None => State::MaxWidth,
            }
        } else {
            State::Process
        }
    }

    fn overflow(&mut self, input: &str) {
        let mut chars = input.char_indices();

        while let Some((i, c)) = chars.next() {
            let char_width = c.width().unwrap_or(0);
            if self.current_width + char_width > self.max_size.width {
                self.lines.range(0, i);
                self.lines.current_slice += 1;
                return;
            }
            self.current_width += char_width;
        }

        self.lines.all(input.len());
        self.lines.current_slice += 1;
    }

    pub fn process(&mut self, input: &str) {
        if !self.check() {
            return;
        }

        if self.line_count == 0 {
            self.line_count = 1;
        }

        if let Wrap::Overflow = self.wrap {
            return self.overflow(input);
        }

        let mut byte = 0;

        let mut chars = input.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if self.line_count > self.max_size.height {
                return;
            }

            let char_width = c.width().unwrap_or(0);
            let state = self.state(c, char_width);

            match state {
                State::ExceedingCharWidth | State::MaxWidth | State::Newline => {
                    self.longest_line = self.longest_line.max(self.current_width);
                    self.current_width = 0;
                    self.lines.range(byte, i);
                    self.lines.newline();
                    self.line_count += 1;
                    byte = i;
                }
                _ => {}
            }

            match state {
                State::Newline => {
                    byte += c.len_utf8();
                    continue;
                }
                State::ExceedingCharWidth => {
                    // Set the current width to max width so it
                    // will force another new line if there are more characters
                    self.current_width = self.max_size.width;
                    byte += c.len_utf8();
                    continue;
                }
                State::WordBoundary(wb, current) => {
                    self.current_width -= wb.width;
                    self.longest_line = self.longest_line.max(wb.width);
                    self.line_count += 1;

                    if current {
                        self.lines.inner.push(Entry::Range(wb.range));
                        self.lines.newline();
                        byte = wb.range.end + wb.skip;
                    } else {
                        let len = self.lines.remove_get_len(wb.entry_index);

                        // if the len is greater than the char len
                        // then there are characters remaining in this slice.
                        if len - wb.skip > 0 {
                            let entry = Entry::Range(Range {
                                slice: wb.range.slice,
                                start: wb.range.end,
                                end: len,
                            });
                            self.lines.insert(wb.entry_index, entry);
                        }
                        self.lines.insert(wb.entry_index, Entry::Newline);
                        self.lines.insert(wb.entry_index, Entry::Range(wb.range));
                    }
                }
                State::MaxWidth => {
                    if c.is_whitespace() && c != '\n' {
                        byte += c.len_utf8();
                    }
                }
                State::Process => {}
            }

            // Consume excess whitespace.
            // This is only applicable during `Normal`
            // word wrapping.
            if let Wrap::Normal = self.wrap {
                match state {
                    State::ExceedingCharWidth | State::MaxWidth | State::Newline => {
                        while let Some((idx, ch)) = chars.next_if(|(_, c)| c.is_whitespace()) {
                            byte = idx + ch.len_utf8();
                        }

                        if c.is_whitespace() {
                            continue;
                        }
                    }
                    State::Process => {}
                    State::WordBoundary(..) => {}
                }
            }

            self.current_width += c.width().unwrap_or(0);

            if let Wrap::Normal = self.wrap {
                if is_word_boundary(c) {
                    let (end, skip) = if c.is_whitespace() {
                        (i, c.len_utf8())
                    } else {
                        (i + c.len_utf8(), 0)
                    };

                    let range = Range {
                        slice: self.lines.current_slice,
                        start: byte,
                        end,
                    };

                    self.word_boundary = Some(WordBoundary::new(
                        self.current_width,
                        self.lines.inner.len(),
                        range,
                        skip,
                    ));
                }
            }
        }

        // If the byte is zero, it means this input was never split,
        // so it's possible to return the entire input.
        if byte == 0 {
            self.lines.all(input.len());
        } else {
            self.lines.range(byte, input.len());
        }

        self.lines.current_slice += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn layout(
        inputs: impl IntoIterator<Item = &'static str>,
        max_width: usize,
        max_height: usize,
    ) -> TextLayout {
        layout_wrapped(inputs, max_width, max_height, Wrap::Normal)
    }

    fn layout_wrapped(
        inputs: impl IntoIterator<Item = &'static str>,
        max_width: usize,
        max_height: usize,
        wrap: Wrap,
    ) -> TextLayout {
        let mut layout = TextLayout::new(Size::new(max_width, max_height), wrap);
        for input in inputs.into_iter() {
            layout.process(input);
        }

        layout
    }

    fn size(
        inputs: impl IntoIterator<Item = &'static str>,
        max_width: usize,
        max_height: usize,
    ) -> Size {
        layout(inputs, max_width, max_height).size()
    }

    fn lines(
        inputs: impl IntoIterator<Item = &'static str>,
        max_width: usize,
        max_height: usize,
        wrap: Wrap,
    ) -> Vec<String> {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        let layout = layout_wrapped(inputs.clone(), max_width, max_height, wrap);

        let mut output = vec![];
        let mut buf = String::new();

        for line in layout.lines.inner {
            match line {
                Entry::All { index, .. } => buf.push_str(&inputs[index]),
                Entry::Range(r) => {
                    let input = &inputs[r.slice];
                    let text = &input[r.start..r.end];
                    buf.push_str(text);
                }
                Entry::Newline => output.push(buf.drain(..).collect::<String>()),
            }
        }

        output.push(buf);

        output
    }

    #[test]
    fn test_size() {
        let expected = Size::new(5, 2);
        let actual = layout(["a", "-1", "2345"], 5, 100).size();
        assert_eq!(expected, actual);
    }

    #[test]
    fn layout_strings() {
        let layout = lines(["a", "-1", "2345"], 5, 100, Wrap::Normal);
        assert_eq!(&layout[0], "a-");
        assert_eq!(&layout[1], "12345");
    }

    #[test]
    fn newlines() {
        let layout = lines(["ab\ncd"], 100, 100, Wrap::Normal);
        assert_eq!(&layout[0], "ab");
        assert_eq!(&layout[1], "cd");
    }

    #[test]
    fn multiple_newlines() {
        let layout = lines(["ab\n\n\ncd"], 100, 100, Wrap::Normal);
        assert_eq!(&layout[0], "ab");
        assert_eq!(&layout[1], "");
        assert_eq!(&layout[2], "");
        assert_eq!(&layout[3], "cd");
    }

    #[test]
    fn break_long_words() {
        let inputs = ["hello suuuuuuperlooooooong hi"];
        let layout = lines(inputs.clone(), 8, 100, Wrap::Normal);
        assert_eq!(&layout[0], "hello");
        assert_eq!(&layout[1], "suuuuuup");
        assert_eq!(&layout[2], "erlooooo");
        assert_eq!(&layout[3], "oong hi");

        let expected = Size::new(8, 4);
        let size = size(inputs, 8, 100);
        assert_eq!(size, expected);
    }

    #[test]
    fn dot_break() {
        let inputs = ["hello.world"];
        let size = size(inputs, 5, 100);
        let expected = Size::new(5, 3);
        assert_eq!(size, expected);
    }

    #[test]
    fn space_break() {
        let inputs = ["hello world"];
        let size = size(inputs.clone(), 5, 100);
        let expected = Size::new(5, 2);
        assert_eq!(size, expected);

        let strings = lines(inputs, 5, 100, Wrap::Normal);
        assert_eq!(&strings[0], "hello");
        assert_eq!(&strings[1], "world");
    }

    #[test]
    fn space_eater() {
        let inputs = ["hello    world"];
        let strings = lines(inputs.clone(), 5, 100, Wrap::Normal);
        assert_eq!(&strings[0], "hello");
        assert_eq!(&strings[1], "world");

        let strings = lines(inputs, 6, 100, Wrap::Normal);
        assert_eq!(&strings[0], "hello ");
        assert_eq!(&strings[1], "world");
    }

    #[test]
    fn truncate_characters() {
        let inputs = ["1🍅2"];
        let expected = Size::new(1, 3);
        let size = size(inputs, 1, 100);
        assert_eq!(size, expected);
    }

    #[test]
    fn hello_how_are_you() {
        let inputs = ["hello how are you"];
        let strings = lines(inputs.clone(), 8, 3, Wrap::Normal);
        assert_eq!(&strings[0], "hello");
        assert_eq!(&strings[1], "how are");
        assert_eq!(&strings[2], "you");
    }

    #[test]
    fn overflow() {
        let inputs = ["hi"];
        let layout = layout_wrapped(inputs, 1, 2, Wrap::Overflow);
        assert_eq!(layout.size(), Size::new(1, 1));

        let strings = lines(inputs.clone(), 1, 2, Wrap::Overflow);
        assert_eq!(&strings[0], "h");
        assert_eq!(strings.len(), 1);
    }

    #[test]
    fn all_in_one_line() {
        let inputs = ["hi"];
        let layout = layout_wrapped(inputs, 2, 2, Wrap::Overflow);
        assert_eq!(layout.size(), Size::new(2, 1));

        let strings = lines(inputs.clone(), 2, 2, Wrap::Overflow);
        assert_eq!(&strings[0], "hi");
        assert_eq!(strings.len(), 1);
    }

    #[test]
    fn most_of_many_overflow() {
        let inputs = ["h", "ia"];
        let layout = layout_wrapped(inputs, 2, 2, Wrap::Overflow);
        assert_eq!(layout.size(), Size::new(2, 1));

        let strings = lines(inputs.clone(), 2, 2, Wrap::Overflow);
        assert_eq!(&strings[0], "hi");
        assert_eq!(strings.len(), 1);
    }
}
