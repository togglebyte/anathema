use std::fmt::Write;

use sillybug::obs;
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
}

impl<'a, 'b, T> Word<'a, 'b, T> {
    pub(crate) fn split(self, max_width: usize) -> (Self, Self) {
        // let lhs = self.slice.split(max_width);
        let rhs = panic!();
    }
}

impl<T: Copy> std::fmt::Display for Word<'_, '_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slice)
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
}

impl<'a, 'b, T> Iterator for Words<'a, 'b, T> {
    type Item = Word<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut width = 0;
        let (start, c) = self.chars.peek()?;
        let mut current = start;
        obs!("curent.byte", current.byte);

        loop {
            let Some((idx, c)) = self.chars.next() else { break };
            if c.is_whitespace() {
                break;
            }
            obs!("char", c);
            current = idx;
            current.byte += c.len_utf8() as u32;
            obs!("curent.byte", current.byte);
            obs!("width", width);
            width += c.width().unwrap_or(0);
            obs!("width", width);
        }

        let slice = Slice::new(start..current, self.chars.slices);
        let word = Word {
            slice,
            width: width as u32,
        };
        return Some(word);
    }
}
