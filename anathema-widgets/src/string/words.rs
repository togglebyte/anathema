use std::fmt::Write;

use unicode_width::UnicodeWidthChar;

use crate::string::chars::{CharIndices, Index};
use crate::string::slice::Slice;

static BOUNDARIES: &[char] = &['.', ',', '!'];

// -----------------------------------------------------------------------------
//   - Word -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Word<'a, 'b, T> {
    pub(crate) slice: Slice<'a, 'b, T>,
    pub(crate) width: u32,
    // Whitespace width
    pub(crate) whitespace_width: u8,
}

impl<'a, 'b, T: Copy> Word<'a, 'b, T> {
    pub(crate) fn split(mut self, max_width: usize) -> (Self, Self) {
        let mut total_width = 0;
        let mut idx = self.slice.start();
        for (i, c) in self.slice.char_indices() {
            let width = c.width().unwrap_or(0);
            idx = i;
            if width + total_width > max_width {
                break;
            }
            total_width += width;
        }

        let rhs_width = self.width - total_width as u32;
        let lhs_width = total_width as u32;
        let (lhs_slice, rhs_slice) = self.slice.split(idx);

        let lhs = Self {
            slice: lhs_slice,
            width: lhs_width,
            whitespace_width: self.whitespace_width,
        };

        let rhs = Self {
            slice: rhs_slice,
            width: rhs_width,
            whitespace_width: 0,
        };

        (lhs, rhs)
    }

    pub fn start(&self) -> Index {
        self.slice.start()
    }

    pub fn end(&self) -> Index {
        self.slice.end()
    }
}

impl<T: Copy> std::fmt::Display for Word<'_, '_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slice)
    }
}

impl<'a, 'b, T> From<Word<'a, 'b, T>> for Slice<'a, 'b, T> {
    fn from(word: Word<'a, 'b, T>) -> Self {
        word.slice
    }
}

// -----------------------------------------------------------------------------
//   - Word iterator -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Words<'a, 'b, T> {
    chars: CharIndices<'a, 'b, T>,
    start: Index,
}

impl<'a, 'b, T> Words<'a, 'b, T> {
    pub fn new(chars: CharIndices<'a, 'b, T>) -> Self {
        Self {
            start: Index::ZERO,
            chars,
        }
    }

    pub(crate) fn slice(&self) -> Slice<'a, 'b, T> {
        self.chars.slice.clone()
    }
}

impl<'a, 'b, T: Copy> Iterator for Words<'a, 'b, T> {
    type Item = Word<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut width = 0;
        let (mut start, mut c) = self.chars.peek()?;

        while c.is_whitespace() {
            self.chars.next()?;
            (start, c) = self.chars.peek()?;
        }

        let mut current = start;

        let whitespace_width = loop {
            let Some((idx, c)) = self.chars.next() else { break 0 };
            if c.is_whitespace() {
                break c.width().unwrap_or(0) as u8;
            }
            current = idx;
            current.byte += c.len_utf8() as u32;
            width += c.width().unwrap_or(0);
        };

        let slice = self.chars.slice.subslice(start..current);
        let word = Word {
            slice,
            width: width as u32,
            whitespace_width,
        };

        Some(word)
    }
}
