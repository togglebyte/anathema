use std::fmt::Write;
use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use crate::string::chars::Index;

#[derive(Debug)]
pub struct Slice<'a, 'b, T> {
    pub(crate) range: Range<Index>,
    slices: &'b [(&'a str, T)],
}

impl<'a, 'b, T> Clone for Slice<'a, 'b, T> {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            slices: self.slices,
        }
    }
}

impl<'a, 'b, T> Slice<'a, 'b, T> {
    pub fn new(range: Range<Index>, slices: &'b [(&'a str, T)]) -> Self {
        let mut range = range;
        Self { range, slices }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }
}

impl<'a, 'b, T: Copy> Iterator for Slice<'a, 'b, T> {
    type Item = (&'a str, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            return None;
        }

        let (slice_idx, byte) = self.range.start.as_usize();
        let (slice, assoc_val) = &self.slices[slice_idx];
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
