use std::iter::Peekable;

use anathema_geometry::Size;
use sillybug::obs;
use unicode_width::UnicodeWidthChar;

use crate::string::chars::{CharIndices, Index};
use crate::string::slice::Slice;
use crate::string::words::{Word, Words};

static BOUNDARIES: &[char] = &['.', ',', '!'];

const NEWLINE: &str = "\n";

// -----------------------------------------------------------------------------
//   - Word boundary -
// -----------------------------------------------------------------------------
#[derive(Debug)]
struct WordBoundary {
    index: Index,
    width: usize,
    skip: u32,
}

impl WordBoundary {
    const ZERO: Self = Self {
        index: Index::ZERO,
        width: 0,
        skip: 0,
    };

    pub const fn new(mut index: Index, width: usize, skip: u32) -> Self {
        index.byte -= skip;
        Self { index, width, skip }
    }

    fn update(&mut self, index: Index, width: usize) {
        self.index = index;
        self.width = width;
    }

    fn debug(&self) {
        obs!("word boundary", "<some>");
        obs!("word boundary byte", self.index.byte);
        obs!("word boundary slice", self.index.slice);
        obs!("word boundary width", self.width);
        obs!("word boundary skip", self.skip);
    }
}

// -----------------------------------------------------------------------------
//   - Lines iterator -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Lines<'a, 'b, T> {
    words: Peekable<Words<'a, 'b, T>>,
    slices: &'b [(&'a str, T)],
    max: Size,
    line: usize,
}

impl<'a, 'b, T> Lines<'a, 'b, T> {
    pub fn new(words: Words<'a, 'b, T>, slices: &'b [(&'a str, T)], max: Size) -> Self {
        Self {
            words: words.peekable(),
            slices,
            max,
            line: 0,
        }
    }
}

impl<'a, 'b, T: Copy + std::fmt::Debug> Iterator for Lines<'a, 'b, T> {
    type Item = Slice<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.words.next()?;
        let mut width = word.width;

        let start = word.slice.range.start;

        let mut last = None::<Word<'_, '_, T>>;

        loop {
            let next_word = match self.words.peek() {
                Some(w) => w,
                None => {
                    let end = last.map(|word| word.slice.range.end).unwrap_or(word.slice.range.end);
                    break Some(Slice::new(start..end, self.slices));
                }
            };

            if width + next_word.width > self.max.width {
                let end = last.map(|word| word.slice.range.end).unwrap_or(word.slice.range.end);
                break Some(Slice::new(start..end, self.slices));
            }

            last = self.words.next();
        }
    }
}
