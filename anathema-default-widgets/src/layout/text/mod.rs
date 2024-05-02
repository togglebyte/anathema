use anathema_geometry::Size;
use anathema_state::CommonVal;
use anathema_store::tree::ValueId;
use anathema_widgets::layout::{Entry, IterEntry, TextIndex, TextSession};

mod overflow;
mod wordbreak;
mod wordwrap;

#[derive(Debug)]
pub struct Line<I> {
    pub iter: I,
    pub width: u32,
}

#[derive(Debug)]
pub struct Lines<'a> {
    text: TextSession<'a>,
    index: TextIndex,
}

impl<'a> Lines<'a> {
    pub fn new(index: TextIndex, text: TextSession<'a>) -> Self {
        Self { text, index }
    }

    pub fn iter(&self) -> impl Iterator<Item = Line<impl Iterator<Item = IterEntry<'_>>>> {
        self.text
            .layout
            .slice(self.index.layout)
            .split(|l| matches!(l, Entry::Newline))
            .map(|lines| {
                let line_width = match &lines[0] {
                    Entry::LineWidth(width) => *width,
                    fine => panic!("invalid entry. something is wrong with text layout: {fine:?}"),
                };

                let iter = lines[1..].iter().map(|entry| match entry {
                    Entry::Str(start, end) => IterEntry::Str(self.text.bytes.word_from(self.index.bytes, *start, *end)),
                    Entry::SetStyle(s) => IterEntry::Style(*s),
                    Entry::LineWidth(_) | Entry::Newline => unreachable!(),
                });

                Line {
                    iter,
                    width: line_width,
                }
            })
    }
}

pub(crate) enum TextLayout<'a> {
    WordWrap(wordwrap::WordWrapLayout<'a>),
    WordBreak(wordbreak::WordBreakLayout<'a>),
    Overflow(overflow::OverflowLayout<'a>),
}

impl<'a> TextLayout<'a> {
    pub fn new(max_size: impl Into<Size>, wrap: Wrap, mut session: TextSession<'a>, index: TextIndex) -> Self {
        session.layout.push(Entry::LineWidth(0));
        match wrap {
            Wrap::Normal => Self::WordWrap(wordwrap::WordWrapLayout::new(max_size, session, index)),
            Wrap::WordBreak => Self::WordBreak(wordbreak::WordBreakLayout::new(max_size, session, index)),
            Wrap::Overflow => Self::Overflow(overflow::OverflowLayout::new(max_size, session, index)),
        }
    }

    pub fn process(&mut self, input: &str) -> ProcessResult {
        match self {
            TextLayout::WordWrap(layout) => layout.process(input),
            TextLayout::WordBreak(layout) => layout.process(input),
            TextLayout::Overflow(layout) => layout.process(input),
        }
    }

    pub fn finish(&mut self) {
        match self {
            TextLayout::WordWrap(layout) => layout.finish(),
            TextLayout::WordBreak(layout) => layout.finish(),
            TextLayout::Overflow(_layout) => (),
        }
    }

    pub(crate) fn size(self) -> Size {
        match self {
            TextLayout::WordWrap(layout) => layout.size(),
            TextLayout::WordBreak(layout) => layout.size(),
            TextLayout::Overflow(layout) => layout.size(),
        }
    }

    pub(crate) fn set_style(&mut self, style: ValueId) {
        match self {
            TextLayout::WordWrap(layout) => layout.set_style(style),
            TextLayout::WordBreak(layout) => layout.set_style(style),
            TextLayout::Overflow(layout) => layout.set_style(style),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ProcessResult {
    Done,
    Continue,
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

impl TryFrom<CommonVal<'_>> for Wrap {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Str(wrap) => match wrap {
                "normal" => Ok(Wrap::Normal),
                "break" => Ok(Wrap::WordBreak),
                "overflow" => Ok(Wrap::Overflow),
                _ => Err(()),
            },
            _ => Err(()),
        }
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

impl TryFrom<CommonVal<'_>> for TextAlignment {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Str(wrap) => match wrap {
                "left" => Ok(TextAlignment::Left),
                "right" => Ok(TextAlignment::Right),
                "centre" | "center" => Ok(TextAlignment::Centre),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[cfg(test)]
pub mod testing {
    use anathema_geometry::Size;
    use anathema_widgets::layout::TextBuffer;

    use super::TextLayout;
    use crate::Wrap;

    pub fn layout(inputs: &[&str], size: impl Into<Size>, wrap: Wrap) -> Size {
        let mut text_buffer = TextBuffer::empty();
        let mut session = text_buffer.new_session();
        let key = session.new_key();
        let mut layout = TextLayout::new(size, wrap, session, key);

        for input in inputs {
            layout.process(input);
        }

        layout.finish();
        layout.size()
    }
}
