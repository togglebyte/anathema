use std::fmt::Write;
use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use crate::string::chars::{CharIndices, Index};

#[derive(Debug)]
pub struct Slice<'a, 'b, T> {
    pub(crate) range: Range<Index>,
    slices: &'b [(&'a str, T)],
}

impl<'a, 'b, T: Copy> Slice<'a, 'b, T> {
    pub fn new(range: Range<Index>, slices: &'b [(&'a str, T)]) -> Self {
        let mut range = range;
        Self { range, slices }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }

    pub(crate) fn first(&self) -> Option<(&'a str, T)> {
        let slice = self.range.start.slice as usize;
        if slice >= self.slices.len() {
            return None;
        }
        let s = self.slices[slice];
        Some(s)
    }

    pub(crate) fn subslice(&self, range: Range<Index>) -> Self {
        Self {
            range,
            slices: self.slices,
        }
    }

    pub(crate) fn start(&self) -> Index {
        self.range.start
    }

    pub(crate) fn end(&self) -> Index {
        self.range.end
    }

    pub(crate) fn char_indices(&self) -> CharIndices<'a, 'b, T> {
        CharIndices::new(self.clone())
    }

    pub(crate) fn split(&self, idx: Index) -> (Self, Self) {
        let mut lhs = self.clone();
        lhs.range.end = idx;
        let mut rhs = self.clone();
        rhs.range.start = idx;
        (lhs, rhs)
    }
}

impl<'a, 'b, T> Clone for Slice<'a, 'b, T> {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            slices: self.slices,
        }
    }
}

impl<'a, 'b, T: Copy> Iterator for Slice<'a, 'b, T> {
    type Item = (&'a str, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            return None;
        }

        let (slice_idx, byte) = self.range.start.as_usize();
        let (slice, assoc_val) = self.slices.get(slice_idx)?;
        let string = &slice[byte..];

        if self.range.end.slice != self.range.start.slice {
            self.range.start.slice += 1;
            self.range.start.byte = 0;
            Some((string, *assoc_val))
        } else {
            // This is the final slice, so take all bytes
            // from .. to self.range.end.byte
            let (_slice, byte) = self.range.end.as_usize();
            let byte = byte - self.range.start.byte as usize;
            self.range.start.byte = self.range.end.byte;
            Some((&string[0..byte], *assoc_val))
        }
    }
}

impl<'a, 'b, T: Copy> std::fmt::Display for Slice<'a, 'b, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (slice, _) in self.clone() {
            write!(f, "{slice}")?;
        }
        Ok(())
    }
}
