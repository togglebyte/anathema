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
use unicode_width::UnicodeWidthChar;

static WRAPPING_CHARS: &[char] = &['.', ';', '!', ','];

const NEWLINE: char = '\n';

pub struct TextLayout {
    lines: Vec<Line>,
}

impl TextLayout {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }
}

struct Line {
    index: u32,
    len: u32,
    pos: Pos,
}

struct WordBreak {
    index: usize,
    byte: usize,
}

impl WordBreak {
    const NONE: Self = Self {
        index: usize::MAX,
        byte: usize::MAX,
    };
}

// -----------------------------------------------------------------------------
//   - This is where the layout is done -
// -----------------------------------------------------------------------------
pub(crate) struct PerformLayout {
    max: Size,
    lines: Vec<Line>,
    last_break: WordBreak,
    current_width: usize,
}

impl PerformLayout {
    pub fn new(max: Size) -> Self {
        Self {
            max,
            lines: vec![],
            last_break: WordBreak::NONE,
            current_width: 0,
        }
    }

    pub fn feed(&mut self, input: &str) {
        for (idx, c) in input.char_indices() {

            // If there is no more available space stop the iteration

            let width = c.width().unwrap_or(0);

            // max = 5
            //
            // hello world -> hello
            //                world
            //
            // hello.world    hello
            //                .
            //                world

            if width + self.current_width > self.max.width as usize {
                self.break_line();
            } else {
                self.current_width += width;
            }

            if c == NEWLINE {
                // Tag this as the last break
                self.break_line();
            }

            if WRAPPING_CHARS.contains(&c) {
                // Tag this as the last break, but include the character
            }

            if c.is_whitespace() {
                // Tag this as the last break, but skip the character
            }
        }
    }

    fn break_line(&mut self) {
    }

    pub(crate) fn finish(&self) -> TextLayout {
        todo!()
    }
}
