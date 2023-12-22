// * Keep building on the left until
//   * there is a different slice       -> push left
//   * there is a word boundary         -> insert word break, focus on right
//   * there are no more inputs         -> push left + wb + right (drain entire tree)
//   * there is a newline char          -> push left + wb + right (drain entire tree)
// * Keep building on the right until
//   * there is a word boundary         -> left = left + wb + right, update wb, focus right
//   * there is a newline char          -> push left + wb + right, then focus left (drain entire tree)
//   * there is no more space           -> push left + wb then left = right, focus left (drain left + wb)
//   * there are no more inputs         -> push left + wb + right (drain entire tree)

use anathema_render::Size;
use anathema_values::{
    impl_dyn_value, Context, DynValue, NodeId, Immediate, Value, ValueExpr, ValueRef,
};
use unicode_width::UnicodeWidthChar;

fn word_break(c: char) -> bool {
    c == '-' || c.is_whitespace()
}

#[derive(Debug)]
enum Drain {
    Left, // including word boundary
    All,
}

#[derive(Debug)]
pub struct Line {
    pub segments: Vec<LineSegment>,
    pub width: usize,
}

impl Line {
    pub fn new(segments: Vec<LineSegment>) -> Self {
        let width = segments.iter().map(|seg| seg.width).sum::<usize>();
        Self { segments, width }
    }
}

#[derive(Debug)]
pub struct LineSegment {
    start: usize,
    end: usize,
    width: usize,
    pub index: usize,
}

impl LineSegment {
    pub fn slice<'a>(&self, s: &'a str) -> &'a str {
        &s[self.start..self.end]
    }

    fn new(start: usize, byte_count: usize, index: usize, width: usize) -> Self {
        Self {
            start,
            end: start + byte_count,
            width,
            index,
        }
    }
}

#[derive(Debug, Default)]
enum Entry {
    #[default]
    Empty,
    Single(LineSegment),
    Many(Vec<LineSegment>, LineSegment),
}

impl Entry {
    fn width(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Single(val) => val.width,
            Self::Many(vals, val) => val.width + vals.iter().map(|v| v.width).sum::<usize>(),
        }
    }

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    fn push(&mut self, val: LineSegment) {
        match self.take() {
            Entry::Empty => *self = Entry::Single(val),
            Entry::Single(mut old) => {
                if old.index == val.index {
                    old.end = val.end;
                    old.width += val.width;
                    *self = Entry::Single(old);
                } else {
                    *self = Entry::Many(vec![old], val)
                }
            }
            Entry::Many(mut vals, mut old) => {
                if old.index == val.index {
                    old.end = val.end;
                    old.width += val.width;
                    *self = Entry::Many(vals, old)
                } else {
                    vals.push(old);
                    *self = Entry::Many(vals, val)
                }
            }
        }
    }

    fn merge(&mut self, other: &mut Entry) {
        if let Entry::Empty = self {
            std::mem::swap(self, other);
            return;
        }

        match other.take() {
            Entry::Empty => (),
            Entry::Single(val) => self.push(val),
            Entry::Many(mut vals, val) => match self.take() {
                Self::Empty => unreachable!(),
                Self::Single(left_val) => {
                    vals.insert(0, left_val);
                    *self = Entry::Many(vals, val)
                }
                Self::Many(mut left_vals, left_val) => {
                    left_vals.push(left_val);
                    left_vals.extend(vals);
                    *self = Self::Many(left_vals, val);
                }
            },
        }
    }

    fn drain(&mut self) -> Vec<LineSegment> {
        match self.take() {
            Entry::Empty => vec![],
            Entry::Single(val) => vec![val],
            Entry::Many(mut vals, val) => {
                vals.push(val);
                vals
            }
        }
    }
}

#[derive(Debug)]
enum Focus {
    Left,
    Right,
}

#[derive(Debug)]
struct Tree {
    left: Entry,
    middle: Option<(LineSegment, bool)>,
    right: Entry,
    focus: Focus,
}

impl Tree {
    fn new() -> Self {
        Self {
            left: Entry::Empty,
            middle: None,
            right: Entry::Empty,
            focus: Focus::Left,
        }
    }

    fn push(&mut self, byte_index: usize, byte_count: usize, slice_index: usize, width: usize) {
        let val = LineSegment::new(byte_index, byte_count, slice_index, width);
        match self.focus {
            Focus::Left => self.left.push(val),
            Focus::Right => self.right.push(val),
        }
    }

    fn set_middle(&mut self, val: LineSegment, is_whitespace: bool) {
        match self.middle.take() {
            None => self.middle = Some((val, is_whitespace)),
            Some((old, _)) => {
                self.left.push(old);
                self.left.merge(&mut self.right);
                self.middle = Some((val, is_whitespace));
            }
        }
    }

    fn drain(&mut self, drain: Drain) -> Line {
        std::mem::swap(&mut self.left, &mut self.right);

        let mut segments = self.right.drain();

        if let Some((val, _)) = self.middle.take() {
            segments.push(val);
        }

        if let Drain::All = drain {
            segments.append(&mut self.left.drain());
        }

        Line::new(segments)
    }
}

#[derive(Debug)]
pub struct TextLayout {
    tree: Tree,
    max_width: usize,
    current_width: usize,
    lines: Vec<Line>,
    // Ignore a line if it contains a singular whitespace
    squash: bool,
    slice_index: usize,
    wrap: Wrap,
}

impl TextLayout {
    pub fn new(max_width: usize, squash: bool, wrap: Wrap) -> Self {
        Self {
            tree: Tree::new(),
            max_width,
            current_width: 0,
            lines: vec![],
            squash,
            slice_index: 0,
            wrap,
        }
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn reset(&mut self, max_width: usize, squash: bool) {
        self.max_width = max_width;
        self.lines.clear();
        self.current_width = 0;
        self.slice_index = 0;
        self.tree = Tree::new();
        self.squash = squash;
    }

    fn process_word_wrap(&mut self, s: &str) {
        for (i, c) in s.char_indices() {
            let width = c.width().unwrap_or(0);

            if width + self.current_width > self.max_width {
                // Squash = remove whitespace that would otherwise
                // be trailing the last character on the left
                let line = if c.is_whitespace() && self.squash {
                    self.tree.drain(Drain::All)
                } else {
                    self.tree.drain(Drain::Left)
                };

                self.tree.focus = Focus::Left;
                self.current_width = self.tree.left.width();

                self.lines.push(line);
                if c.is_whitespace() && self.squash {
                    continue;
                }
            }

            self.current_width += width;

            match c {
                '\n' => {
                    let line = self.tree.drain(Drain::All);
                    self.lines.push(line);
                    self.tree.focus = Focus::Left;
                }
                _ if word_break(c) => {
                    self.tree.set_middle(
                        LineSegment::new(i, c.len_utf8(), self.slice_index, width),
                        c.is_whitespace(),
                    );
                    self.tree.focus = Focus::Right;
                }
                _ => self.tree.push(i, c.len_utf8(), self.slice_index, width),
            }
        }

        self.slice_index += 1;
    }

    fn process_word_break(&mut self, s: &str) {
        for (i, c) in s.char_indices() {
            let width = c.width().unwrap_or(0);
            if width + self.current_width > self.max_width {
                let line = self.tree.drain(Drain::Left);
                self.lines.push(line);
                self.current_width = 0;
            }
            self.current_width += width;
            self.tree.push(i, c.len_utf8(), self.slice_index, width);
        }
    }

    fn process_overflow(&mut self, s: &str) {
        for (i, c) in s.char_indices() {
            let width = c.width().unwrap_or(0);
            self.tree.push(i, c.len_utf8(), self.slice_index, width);
        }
    }

    pub fn size(&self) -> Size {
        Size {
            height: self.lines.len(),
            width: self.lines.iter().map(|line| line.width).max().unwrap_or(0),
        }
    }

    pub fn finish(&mut self) {
        let line = self.tree.drain(Drain::All);
        self.lines.push(line);
    }

    pub fn process(&mut self, s: &str) {
        match self.wrap {
            Wrap::Normal => self.process_word_wrap(s),
            Wrap::WordBreak => self.process_word_break(s),
            Wrap::Overflow => self.process_overflow(s),
        }
    }
}

/// Word wrapping strategy
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Wrap {
    /// Normal word wrapping. This will break text on hyphen and whitespace.
    /// Trailing whitespace is consumed if it would cause a line break.
    #[default]
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
