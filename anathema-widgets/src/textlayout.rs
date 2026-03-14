// Layout is created as a series of lines of text that can span multiple widgets.
// A single widget is therefore responsible for multiple regions to style, unlike
// most widgets that will style a single region based on its position and size.
//
// Because the text already exists on the widget it doesn't make sense to clone it,
// so the text layout can be made up of indices for each line.
//
// So with that in mind: text layout can be multiple regions per widget.

// -----------------------------------------------------------------------------
//   - Rules for a newline -
//   * A newline character
//   * Punctuation chars or whitespace
// -----------------------------------------------------------------------------

use anathema_geometry::{Pos, Size};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static WRAPPING_CHARS: &[char] = &['.', ';', '!', ','];

const NEWLINE: char = '\n';

#[derive(Debug)]
pub(crate) struct TextLayout {
    pub(crate) segments: Vec<Segment>,
}

impl TextLayout {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Segment {
    instructions: Vec<Instruction>,
}
impl Segment {
    fn new() -> Self {
        Self { instructions: vec![] }
    }

    pub(crate) fn instructions(&self) -> impl Iterator<Item = Instruction> {
        self.instructions.iter().cloned()
    }

    fn newline(&mut self) {
        self.instructions.push(Instruction::Newline);
    }

    fn split_last_print(&mut self, byte_index: usize, skip: usize) {
        let Some(Instruction::Print(range)) = self.instructions.last_mut() else { unreachable!() };

        let lhs = range.start..byte_index;
        let rhs = byte_index + skip .. range.end;
        *range = lhs;
        self.instructions.push(Instruction::Newline);
        self.instructions.push(Instruction::Print(rhs));
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Instruction {
    Newline,
    Print(std::ops::Range<usize>),
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Position {
    segment: usize,
    start: usize,
    end: usize,
}

impl Position {
    const NONE: Self = Self {
        segment: usize::MAX,
        start: usize::MAX,
        end: usize::MAX,
    };
    const ZERO: Self = Self {
        segment: 0,
        start: 0,
        end: 0,
    };

    fn make_word_boundary(&self, skip: usize) -> WordBoundary {
        WordBoundary {
            segment: self.segment,
            byte_index: self.end,
            skip,
        }
    }
}

#[derive(Debug, PartialEq)]
struct WordBoundary {
    segment: usize,
    byte_index: usize,
    skip: usize,
}

impl WordBoundary {
    const NONE: Self = Self {
        segment: usize::MAX,
        byte_index: usize::MAX,
        skip: usize::MAX,
    };
}

// -----------------------------------------------------------------------------
//   - This is where the layout is done -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct PerformLayout {
    max: Size,
    segments: Vec<Segment>,
    last_boundary: WordBoundary,
    current: Position,
    current_width: usize,
    current_seg: Segment,
}

impl PerformLayout {
    pub fn new(max: Size) -> Self {
        Self {
            max,
            segments: vec![],
            last_boundary: WordBoundary::NONE,
            current: Position::ZERO,
            current_width: 0,
            current_seg: Segment::new(),
        }
    }

    pub fn feed(&mut self, input: &str) {
        // Create a new segment for the string

        for (idx, c) in input.char_indices() {
            // If there is no more available space stop the iteration

            let width = c.width().unwrap_or(0);
            self.current.end = idx;

            // max = 5
            //
            // hello world -> hello
            //                world
            //
            // hello.world    hello
            //                .
            //                world

            if width + self.current_width > self.max.width as usize {
                self.newline();
            } else {
                self.current_width += width;
            }

            if c == NEWLINE {
                // Tag this as the last break
                if self.current != Position::ZERO {}
                self.last_boundary = WordBoundary::NONE;
                continue;
            }

            if c.is_whitespace() {
                // Tag this as the last break, but skip the character
                self.last_boundary = self.current.make_word_boundary(c.len_utf8());
                continue;
            }

            if WRAPPING_CHARS.contains(&c) {
                // Tag this as the last break, but include the character
                self.last_boundary = self.current.make_word_boundary(0);
            }

            self.current.end += c.len_utf8();
        }

        self.next_segment();
    }

    pub(crate) fn finish(&mut self) -> TextLayout {
        self.segments.push(std::mem::take(&mut self.current_seg));
        TextLayout {
            segments: std::mem::take(&mut self.segments),
        }
    }

    fn newline(&mut self) {
        if self.last_boundary != WordBoundary::NONE {
            let segment = &mut self.segments[self.last_boundary.segment];
            segment.split_last_print(self.last_boundary.byte_index, self.last_boundary.skip);
            self.last_boundary = WordBoundary::NONE;
        }
        self.current_seg.instructions.push(Instruction::Newline);
    }

    fn next_segment(&mut self) {
        self.current_seg
            .instructions
            .push(Instruction::Print(self.current.start..self.current.end));
        self.segments.push(std::mem::take(&mut self.current_seg));
        self.current.segment += 1;
        self.current.start = 0;
        self.current.end = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn word_wrap() {
        let max = Size::new(7, 2);
        let mut layout = PerformLayout::new(max);

        let input = ["he", "l", "lo w", "o", "rldyworld"];

        for i in input {
            layout.feed(i);
        }

        let layout = layout.finish();

        for (seg, input) in layout.segments.iter().zip(input) {
            for inst in seg.instructions() {
                match inst {
                    Instruction::Newline => println!(""),
                    Instruction::Print(range) => print!("{}", &input[range]),
                }
            }
        }

        panic!();
    }
}
