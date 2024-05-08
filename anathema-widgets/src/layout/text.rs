use std::ops::{AddAssign, Deref};

use anathema_geometry::Size;
use anathema_state::CommonVal;
use anathema_store::buffer::{Buffer, Session, SliceIndex};
use anathema_store::tree::ValueId;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Word wrapping strategy
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Wrap {
    /// Normal word wrapping. This will break text on hyphen and whitespace.
    /// Trailing whitespace is consumed if it would cause a line break.
    #[default]
    Normal,
    /// Insert a newline in the middle of any text
    WordBreak,
}

impl Wrap {
    /// Returns true if word wrapping is enabled (self == Self::Normal)
    pub fn is_word_wrap(&self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl TryFrom<CommonVal<'_>> for Wrap {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Str(wrap) => match wrap {
                "normal" => Ok(Wrap::Normal),
                "break" => Ok(Wrap::WordBreak),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct Width(usize);

impl Width {
    const ZERO: Self = Self(0);

    fn swap(&mut self, mut new_value: usize) -> u16 {
        std::mem::swap(&mut self.0, &mut new_value);
        new_value as u16
    }
}

impl Deref for Width {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign<usize> for Width {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessResult {
    Continue,
    Break,
}

/// A shared storage of byte, layout and line data for string layout.
pub struct StringStorage {
    bytes: Buffer<u8>,
    layout: Buffer<(u32, Entry)>,
    lines: Buffer<LineEntry>,
}

impl StringStorage {
    /// Create a new instance of string storage.
    pub fn new() -> Self {
        Self {
            bytes: Buffer::empty(),
            layout: Buffer::empty(),
            lines: Buffer::empty(),
        }
    }

    /// Create a new session for text layout.
    pub fn new_session(&mut self) -> StringSession<'_> {
        StringSession {
            bytes: self.bytes.new_session(),
            layout: self.layout.new_session(),
            lines: self.lines.new_session(),
        }
    }

    /// Clear the storage (this is done between frames to reuse the memory).
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.layout.clear();
        self.lines.clear();
    }
}

/// A temporary session for text layout.
pub struct StringSession<'buf> {
    bytes: Session<'buf, u8>,
    layout: Session<'buf, (u32, Entry)>,
    lines: Session<'buf, LineEntry>,
}

impl<'buf> StringSession<'buf> {
    /// Create a new instance of string layout given a max size and rules
    /// on how to handle word wrapping.
    pub fn new_layout(&mut self, max: Size, wrap: Wrap) -> Strings<'_, 'buf> {
        let byte_key = self.bytes.next_slice();
        let bytes = Bytes::new(&mut self.bytes, byte_key);
        let layout_key = self.layout.next_slice();
        let layout = Layout::new(&mut self.layout, layout_key);
        let lines_key = self.lines.next_slice();
        let lines = Lines::new(&mut self.lines, lines_key);

        Strings {
            bytes,
            layout,
            lines,
            chomper: Chomper::Continuous(0),
            current_width: Width::ZERO,
            wrap,
            max,
            line: 0,
            size: Size::new(0, 1),
            frozen: false,
        }
    }

    /// Access the laid out strings.
    /// See [`Strings`] for example.
    pub fn lines(&self, key: LayoutKey) -> impl Iterator<Item = Line<impl Iterator<Item = Segment<'_>>>> {
        let lines = self.lines.slice(key.layout).split(|e| *e == LineEntry::Newline);
        let bytes = self.bytes.slice(key.bytes);

        lines.map(|entries| {
            let LineEntry::Width(width) = entries[0] else { unreachable!() };
            Line {
                width,
                entries: entries[1..].iter().map(|e| match e {
                    LineEntry::Str(from, to) => Segment::Str(
                        std::str::from_utf8(&bytes[*from as usize..*to as usize])
                            .expect("only strings written to the byte store"),
                    ),
                    LineEntry::SetStyle(style) => Segment::SetStyle(*style),
                    LineEntry::Width(_) | LineEntry::Newline => unreachable!("consumed already"),
                }),
            }
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum Entry {
    Newline,
    LineWidth(u16),
    Style(ValueId),
}

/// Represents a line containing the width and the segments
#[derive(Debug)]
pub struct Line<I> {
    pub width: u16,
    pub entries: I,
}

/// A line segment.
#[derive(Debug)]
pub enum Segment<'a> {
    /// Set the style
    SetStyle(ValueId),
    /// String slice
    Str(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum LineEntry {
    Width(u16),
    Str(u32, u32),
    SetStyle(ValueId),
    Newline,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LayoutKey {
    layout: SliceIndex,
    bytes: SliceIndex,
}

impl From<(SliceIndex, SliceIndex)> for LayoutKey {
    fn from((layout, bytes): (SliceIndex, SliceIndex)) -> Self {
        Self { layout, bytes }
    }
}

#[derive(Debug)]
struct Bytes<'a, 'buf> {
    inner: &'a mut Session<'buf, u8>,
    key: SliceIndex,
}

impl<'a, 'buf> Bytes<'a, 'buf> {
    pub fn new(inner: &'a mut Session<'buf, u8>, key: SliceIndex) -> Self {
        Self { inner, key }
    }

    fn str(&self, offset: usize, index: usize) -> &str {
        let slice = &self.inner.slice(self.key);
        std::str::from_utf8(&slice[offset..index]).expect("only valid strings here")
    }

    fn extend(&mut self, bytes: impl Iterator<Item = u8>) {
        self.inner.extend(bytes);
    }

    fn len(&self) -> u32 {
        self.inner.slice(self.key).len() as u32
    }

    fn slice(&self) -> &[u8] {
        self.inner.slice(self.key)
    }

    fn truncate(&mut self, index: usize) {
        self.inner.truncate(self.key, index);
    }
}

#[derive(Debug)]
struct Layout<'a, 'buf> {
    inner: &'a mut Session<'buf, (u32, Entry)>,
    key: SliceIndex,
}

impl<'a, 'buf> Layout<'a, 'buf> {
    pub fn new(inner: &'a mut Session<'buf, (u32, Entry)>, key: SliceIndex) -> Self {
        Self { inner, key }
    }

    fn push(&mut self, index: u32, entry: Entry) {
        self.inner.push((index, entry));
    }

    fn as_slice(&self) -> &[(u32, Entry)] {
        self.inner.slice(self.key)
    }
}

#[derive(Debug)]
struct Lines<'a, 'buf> {
    inner: &'a mut Session<'buf, LineEntry>,
    key: SliceIndex,
}

impl<'a, 'buf> Lines<'a, 'buf> {
    pub fn new(inner: &'a mut Session<'buf, LineEntry>, key: SliceIndex) -> Self {
        Self { inner, key }
    }

    pub fn push(&mut self, entry: LineEntry) {
        self.inner.push(entry);
    }

    fn pop(&mut self) {
        self.inner.pop();
    }
}

/// Perform text layout
/// ```
/// # use anathema_widgets::layout::text::*;
/// # use anathema_geometry::Size;
/// let mut string_storage = StringStorage::new();
/// let mut session = string_storage.new_session();
///
/// let mut text = session.new_layout(Size::new(10, 10), Wrap::Normal);
/// text.add_str("he");
/// text.add_str("ll");
/// text.add_str("o world");
/// let (key, size) = text.finish();
///
/// let lines = session.lines(key);
/// ```
#[derive(Debug)]
pub struct Strings<'a, 'buf> {
    layout: Layout<'a, 'buf>,
    bytes: Bytes<'a, 'buf>,
    lines: Lines<'a, 'buf>,
    chomper: Chomper,
    current_width: Width,
    wrap: Wrap,
    max: Size,
    // Byte index where the current line starts
    line: usize,
    size: Size,
    frozen: bool,
}

impl<'buf> Strings<'_, 'buf> {
    fn line(&self, index: usize) -> &str {
        self.bytes.str(self.line, index)
    }

    pub fn add_str(&mut self, s: &str) -> ProcessResult {
        if self.frozen {
            return ProcessResult::Break;
        }
        for word in s.split_inclusive(char::is_whitespace) {
            self.bytes.extend(word.bytes());
            for c in word.chars() {
                if let res @ ProcessResult::Break = self.chomp(c) {
                    self.bytes.truncate(self.chomper.index());
                    self.freeze();
                    return res;
                }
            }
        }

        ProcessResult::Continue
    }

    fn chomp(&mut self, c: char) -> ProcessResult {
        let width = c.width().unwrap_or(0);

        // NOTE
        // Special case: the character is too wide to ever fits so it's removed
        if width > self.max.width {
            for _ in 0..c.len_utf8() {
                self.bytes.inner.pop();
            }
            return ProcessResult::Continue;
        }

        // NOTE
        // If newline characters are handled then pop the bytes and insert a newline
        if c == '\n' {
            self.bytes.inner.pop();

            if self.size.height >= self.max.height {
                return ProcessResult::Break;
            }
            self.newline();
            return ProcessResult::Continue;
        }

        // NOTE
        // If the trailing whitespace should be removed do so here
        while width + *self.current_width > self.max.width {
            if c.is_whitespace() {
                for _ in 0..c.len_utf8() {
                    self.bytes.inner.pop();
                }
                return ProcessResult::Continue;
            }

            if self.size.height >= self.max.height {
                return ProcessResult::Break;
            }

            self.newline();
        }

        self.chomper.chomp(c, self.wrap);
        self.current_width += width;

        ProcessResult::Continue
    }

    fn newline(&mut self) {
        self.size.height += 1;
        self.update_width();
        self.line = match self.chomper {
            Chomper::Continuous(idx) => {
                self.layout
                    .push(idx as u32, Entry::LineWidth(self.current_width.swap(0)));
                self.layout.push(idx as u32, Entry::Newline);
                idx
            }
            Chomper::WordBoundary(wb, idx) => {
                let diff = self.line(idx).width() - self.line(wb).width();
                let width = *self.current_width - diff;
                self.layout.push(wb as u32, Entry::LineWidth(width as u16));
                self.layout.push(wb as u32, Entry::Newline);
                let _ = self.current_width.swap(diff);
                self.chomper = Chomper::Continuous(idx);
                wb
            }
        };
    }

    pub fn set_style(&mut self, style: ValueId) {
        let index = self.bytes.len();
        self.layout.push(index, Entry::Style(style));
    }

    pub fn finish(mut self) -> (LayoutKey, Size) {
        self.layout
            .inner
            .slice_mut(self.layout.key)
            .sort_by(|a, b| a.0.cmp(&b.0));

        let slice = self.bytes.slice();
        let last_line = self.line(slice.len());
        let last_line_width = last_line.width();
        self.layout
            .push(slice.len() as u32, Entry::LineWidth(last_line_width as u16));

        // Write the entries as lines
        let mut from = 0;
        for line in self.layout.as_slice().split(|e| matches!(e.1, Entry::Newline)) {
            // Find the line width (always the last entry)
            let width = match line.last() {
                Some((_, Entry::LineWidth(w))) => *w,
                _ => unreachable!("the last entry is always the line width"),
            };

            self.lines.push(LineEntry::Width(width));

            for (i, entry) in line {
                // Don't bother adding a string entry for an empty string
                if from != *i {
                    self.lines.push(LineEntry::Str(from, *i));
                }

                from = *i;

                match entry {
                    Entry::Style(style) => self.lines.push(LineEntry::SetStyle(*style)),
                    Entry::LineWidth(_) => {}
                    Entry::Newline => unreachable!("consumed by the split"),
                }
            }
            self.lines.push(LineEntry::Newline);
        }

        self.lines.pop();

        let key = (self.lines.key, self.bytes.key).into();
        self.update_width();
        if self.size.width == 0 {
            self.size = Size::ZERO;
        }
        (key, self.size)
    }

    fn update_width(&mut self) {
        self.size.width = self.size.width.max(*self.current_width);
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

#[derive(Debug)]
enum Chomper {
    Continuous(usize),
    WordBoundary(usize, usize),
}

impl Chomper {
    fn index(&self) -> usize {
        match self {
            Chomper::Continuous(idx) | Chomper::WordBoundary(_, idx) => *idx,
        }
    }

    fn chomp(&mut self, c: char, wrap: Wrap) {
        let c_len = c.len_utf8();

        if c.is_whitespace() && wrap.is_word_wrap() {
            match self {
                Chomper::Continuous(idx) | Chomper::WordBoundary(_, idx) => {
                    *self = Self::WordBoundary(*idx + c_len, *idx + c_len);
                    return;
                }
            }
        }

        match self {
            Self::Continuous(idx) => *idx += c_len,
            Self::WordBoundary(_, idx) => *idx += c_len,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_layout(max: Size, input: &[&str], expected: &str, wrap: Wrap) {
        let mut string_storage = StringStorage::new();
        let mut session = string_storage.new_session();
        let mut strings = session.new_layout(max, wrap);

        for i in input {
            if let ProcessResult::Break = strings.add_str(i) {
                break;
            }
        }

        let (key, _size) = strings.finish();

        let lines = session.lines(key);

        let mut output = String::new();
        for line in lines {
            for e in line.entries {
                match e {
                    Segment::SetStyle(_) => todo!(),
                    Segment::Str(s) => output.push_str(s),
                }
            }
            output.push('\n');
        }
        output.pop();

        assert_eq!(&output, expected);
    }

    #[test]
    fn word_wrapping_layout() {
        let inputs: &[(&[&str], &str)] = &[
            (&["a\nb\nc"], "a\nb\nc"),
            (&[" 12", "345　12", "345 "], " \n12345\n12345"),
            (&[" 🐇🐇🐇", "🐇🐇 12", "345 "], " \n🐇🐇\n🐇🐇\n🐇 \n12345"),
            (&["1", "23", "45 12", "345 "], "12345\n12345"),
            (&["12345 abcde "], "12345\nabcde"),
            (&["onereallylongword"], "onere\nallyl\nongwo\nrd"),
            (&["ahello do the"], "ahell\no do \nthe"),
            (&["hello do the"], "hello\ndo \nthe"),
        ];

        for (input, expected) in inputs {
            test_layout(Size::new(5, 10), input, expected, Wrap::Normal);
        }
    }

    #[test]
    fn outliers() {
        let inputs: &[(&[&str], &str)] = &[
            (&["🐇"], ""),
            (&["\n"], "\n"),
            (&["\n\n\n"], "\n\n\n"),
            (&["abc"], "a\nb\nc"),
        ];

        for (input, expected) in inputs {
            test_layout(Size::new(1, 10), input, expected, Wrap::Normal);
        }
    }

    #[test]
    fn layout_size() {
        let inputs: &[(&[&str], &str)] = &[(&["123456789"], "123\n456")];

        for (input, expected) in inputs {
            test_layout(Size::new(3, 2), input, expected, Wrap::Normal);
        }
    }

    #[test]
    fn word_breaking_layout() {
        let inputs: &[(&[&str], &str)] = &[(&["123 4567"], "123 4\n567")];

        for (input, expected) in inputs {
            test_layout(Size::new(5, 3), input, expected, Wrap::WordBreak);
        }
    }

    #[test]
    fn freeze_layout() {
        let mut string_storage = StringStorage::new();
        let mut session = string_storage.new_session();
        let mut strings = session.new_layout(Size::new(100, 10), Wrap::Normal);

        assert_eq!(strings.add_str("abc"), ProcessResult::Continue);
        strings.freeze();
        assert_eq!(strings.add_str("abc"), ProcessResult::Break);
    }
}
