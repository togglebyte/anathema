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
}

// -----------------------------------------------------------------------------
//   - Lines iterator -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Lines<'a, 'b, T: Copy> {
    words: Words<'a, 'b, T>,
    max: Size,
    line: usize,
    current: Option<Word<'a, 'b, T>>,
}

impl<'a, 'b, T: Copy> Lines<'a, 'b, T> {
    pub fn new(words: Words<'a, 'b, T>, max: Size) -> Self {
        Self {
            words,
            max,
            line: 0,
            current: None,
        }
    }
}

impl<'a, 'b, T: Copy + std::fmt::Debug> Iterator for Lines<'a, 'b, T> {
    type Item = Slice<'a, 'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line == self.max.height as usize {
            return None;
        }

        // Loop
        // * find the next word
        // * fit it
        // * update the end index
        // repeat until it no longer fits, then make the end index the start index
        // and go again

        // Look at the next word.
        // Does it fit?
        // Yes -> Send it
        // No -> Make a new line
        //  Does it fit?
        //  Yes -> Send it
        //  No -> Split the word by max width

        let word = match self.current.take() {
            // if the word is longer than max width then split the word
            Some(word) if word.width > self.max.width => {
                let (lhs, rhs) = word.split(self.max.width as usize);
                obs!("lhs", format!("{lhs}"));
                obs!("rhs", format!("{rhs}"));
                self.current = Some(rhs);
                self.line += 1;
                return Some(lhs.into());
            }
            Some(word) => word,
            None => {
                self.current = Some(self.words.next()?);
                return self.next();
            }
        };

        let mut width = word.width + word.whitespace_width as u32;
        obs!("word", format!("{word}"));

        let start = word.start();
        let mut end = word.end();
        while let Some(word) = self.words.next() {
            obs!("word", format!("{word}"));
            width += word.width;
            if width > self.max.width {
                self.current = Some(word);
                break;
            }
            width +=  word.whitespace_width as u32;

            end = word.end();
        }

        let range = start..end;
        self.line += 1;
        Some(self.words.slice().subslice(range))
    }
}
