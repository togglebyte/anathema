use std::ops::Range;

use anathema_geometry::Size;

pub use crate::string::chars::{CharIndices, Index};
use crate::string::lines::Lines;
use crate::string::slice::Slice;
use crate::string::words::Words;

mod chars;
mod lines;
mod slice;
mod words;

pub struct SegString<'a, T> {
    inner: Vec<(&'a str, T)>,
}

impl<'a, T: Copy> SegString<'a, T> {
    pub fn new(s: &'a str, val: T) -> Self {
        Self { inner: vec![(s, val)] }
    }

    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    pub fn push(&mut self, segment: &'a str, assoc: T) {
        self.inner.push((segment, assoc));
    }

    pub fn lines<'b>(&'b self, max: Size) -> Lines<'a, 'b, T> {
        let words = self.words();
        Lines::new(words, max)
    }

    pub fn words<'b>(&'b self) -> Words<'a, 'b, T> {
        Words::new(self.as_slice().char_indices())
    }

    pub fn as_slice<'b>(&'b self) -> Slice<'a, 'b, T> {
        let last = self.inner.len() - 1;
        let range = Index::ZERO..Index::new(self.inner.len() as u32, self.inner[last].0.len() as u32);
        Slice::new(range, &self.inner)
    }

    pub fn slice<'b>(&'b self, range: Range<Index>) -> Slice<'a, 'b, T> {
        Slice::new(range, &self.inner)
    }
}

impl<'a, T: Copy> FromIterator<(&'a str, T)> for SegString<'a, T> {
    fn from_iter<V: IntoIterator<Item = (&'a str, T)>>(iter: V) -> Self {
        let mut inst = Self::empty();

        for (string, val) in iter {
            inst.push(string, val);
        }

        inst
    }
}

impl<T> std::fmt::Display for SegString<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (s, _) in &self.inner {
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn slice() {
        let string: SegString<'_, _> = [("hello", ()), (" ", ()), ("world", ()), (".", ())]
            .into_iter()
            .collect();

        let range = Index::new(0, 2)..Index::new(0, 3);

        let slice = string.slice(range);

        println!("{slice}");
        panic!();
    }

    #[test]
    fn iter_chars() {
        let string: SegString<'_, _> = [("hello", ()), (" ", ()), ("world", ()), (".", ())]
            .into_iter()
            .collect();

        for (index, c) in string.char_indices() {
            println!("{c}: {index:?}");
        }

        println!("");

        panic!();
    }

    #[test]
    fn lines() {
        let string: SegString<'_, _> = [("hello", ()), ("x", ()), ("world", ()), (".", ())]
            .into_iter()
            .collect();

        let string: SegString<'_, _> = [("hi", ())].into_iter().collect();

        let mut lines = string.lines(Size::new(5, 10));

        for line in &mut lines {
            eprintln!("{line}");
        }

        panic!("")
        // panic!("{lines:#?}")
    }
}
