#![deny(missing_docs)]
//! A string type made up of string slices.
//!
//! ```ignore
//! use anathema::antstring::{AntString, Contains, Find};
//!
//! #[derive(Debug)]
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! let input = [(&Color::Red, "012"), (&Color::Green, "34"), (&Color::Blue, "5")];
//! let string = AntString::with_annotations(&input);
//!
//! assert!(string.contains('3'));
//! assert_eq!(string.find('3').unwrap(), 3);
//!
//! for (color, c) in string.annotated_chars() {
//!     eprintln!("{c} [{color:?}]");
//! }
//!
//! ```
//!
//! To use [`AntString::find`] and [`AntString::contains`]
//! import `antstring::{Find, Contains}`.
use std::fmt;
use std::iter::Rev;
use std::str::Bytes as StdBytes;
use std::str::CharIndices as StdCharIndices;
use std::str::Chars as StdChars;

use unicode_width::UnicodeWidthStr;

mod contains;
mod find;
mod fromrange;
mod sealed;

use fromrange::FromRange;

pub use contains::Contains;
pub use find::Find;

// -----------------------------------------------------------------------------
//     - Magic string -
// -----------------------------------------------------------------------------
/// A string made up of string slices.
#[derive(Clone)]
pub struct AntString<'a, T> {
    inner: Vec<(&'a T, &'a str)>,
}

impl<'a> AntString<'a, ()> {
    /// Create an annotated string using `()` as the annotation.
    /// This is useful if you don't really care about the annotation.
    pub fn new(inner: impl AsRef<[&'a str]>) -> Self {
        let inner = inner.as_ref().iter().map(|s| (&(), *s)).collect();
        Self { inner }
    }
}

impl<'a, T> AntString<'a, T> {
    /// Create a new instance of a `AntString` from tuples of annotations and string slices.
    pub fn with_annotations(inner: impl AsRef<[(&'a T, &'a str)]>) -> Self {
        Self { inner: inner.as_ref().to_owned() }
    }

    /// The total length of the string in bytes
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.iter().map(|(_, s)| s.len()).sum()
    }

    /// Returns true if this string has a length of zero, otherwise false
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator over the bytes of the inner string slices
    #[cfg(test)]
    #[must_use]
    pub fn bytes(&'a self) -> Bytes<'a, impl Iterator<Item = &'a str>> {
        Bytes::new(self.inner.iter().map(|(_annotation, slice)| *slice))
    }

    /// An iterator over the characters of the inner string slices
    #[cfg(test)]
    #[must_use]
    pub fn chars(&'a self) -> Chars<'a, impl Iterator<Item = &'a str>> {
        Chars::new(self.inner.iter().map(|(_annotation, slice)| *slice))
    }

    /// An iterator over the characters of the inner string slices, in reverse
    #[must_use]
    pub fn chars_rev(&'a self) -> CharsRev<'a, impl Iterator<Item = &'a str>> {
        CharsRev::new(self.inner.iter().rev().map(|(_annotation, slice)| *slice))
    }

    /// An iterator over the characters and their index (byte position) of the inner string slices
    #[must_use]
    pub fn char_indices(&'a self) -> CharIndices<'a, impl Iterator<Item = &'a str>> {
        CharIndices::new(self.inner.iter().map(|(_annotation, slice)| *slice))
    }

    /// Get a substring
    pub fn get(&self, range: impl FromRange) -> Self {
        let mut substring = Self { inner: self.inner.clone() };
        substring.truncate(range);
        substring
    }

    /// Split the string in two:
    #[must_use]
    pub fn split_at(&self, index: usize) -> (Self, Self) {
        let (slice_index, index) = self.index(index);

        let left = {
            let mut v = self.inner[..=slice_index].to_vec();
            let (annotation, slice) = &v[slice_index];
            let last = &slice[..index];

            if last.is_empty() {
                v.pop();
            } else {
                v[slice_index] = (annotation, last);
            }

            v
        };

        let right = {
            let mut v = self.inner[slice_index..].to_vec();
            let (annotation, slice) = &v[0]; //[index..];
            let first = &slice[index..];

            if first.is_empty() {
                v.remove(0);
            } else {
                v[0] = (annotation, first);
            }

            v
        };

        (Self::with_annotations(left), Self::with_annotations(right))
    }

    /// Split the string on newline characters,
    /// and consume the newline char
    pub fn lines(self) -> Lines<'a, T> {
        Lines::new(self)
    }

    /// Cut out a chunk of the string
    pub fn remove(&mut self, range: impl FromRange) {
        let (start, end) = range.into_start_end(self.len());
        let (mut left, right) = self.split_at(end);
        left.truncate(..start);
        left.concat(right);
        *self = left;
    }

    /// Insert a string slice at a given position
    #[cfg(test)]
    pub fn insert(&mut self, pos: usize, new_slice: &'a str) {
        let (mut slice_index, byte_index) = self.index(pos);
        let (annotation, slice) = &self.inner.remove(slice_index);
        let left = (*annotation, &slice[..byte_index]);
        let right = (*annotation, &slice[byte_index..]);

        if !left.1.is_empty() {
            self.inner.insert(slice_index, left);
            slice_index += 1;
        }
        self.inner.insert(slice_index, (left.0, new_slice));
        slice_index += 1;
        if !right.1.is_empty() {
            self.inner.insert(slice_index, right);
        }
    }

    /// Join two [`AntString`] into one
    pub fn concat(&mut self, mut right: AntString<'a, T>) {
        self.inner.append(&mut right.inner);
    }

    /// Trim any white space from the start and the end of the string.
    /// Unlike [`&str`] this does not work for RTL.
    pub fn trim(&mut self) {
        self.trim_start();
        self.trim_end();
    }

    /// Trim the start of the string from any white space characters.
    /// This will not work correctly with RTL
    pub fn trim_start(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut byte_index = 0;

        // If the first character is not a whitespace character then bail
        if !self.inner.get(0).and_then(|(_, s)| s.chars().next()).unwrap_or(' ').is_whitespace() {
            return;
        }

        for (_, slice) in &self.inner {
            let trimmed = slice.trim_start();
            if trimmed.is_empty() {
                byte_index += slice.len();
                continue;
            }
            byte_index += slice.len() - trimmed.len();
            break;
        }

        self.truncate(byte_index..);
    }

    /// Trim the end of the string from any white space characters.
    /// This will not work correctly with RTL
    pub fn trim_end(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut byte_index = self.len();
        // If the last character is not a whitespace character then bail
        if !self.inner.iter().last().and_then(|(_, s)| s.chars().last()).unwrap_or(' ').is_whitespace() {
            return;
        }

        for (_, slice) in self.inner.iter().rev() {
            let trimmed = slice.trim_end();
            if trimmed.is_empty() {
                byte_index -= slice.len();
                continue;
            }

            byte_index -= slice.len() - trimmed.len();
            break;
        }

        self.truncate(..byte_index);
    }

    /// Get a [`AntString`] from a range.
    /// ```ignore
    /// use anathema::antstring::AntString;
    /// let input = ["012", "345"];
    /// let mut string = AntString::new(&input);
    /// // Exclusive range
    /// string.truncate(1..5);
    /// assert_eq!(string.to_string(), "1234".to_string());
    ///
    /// // Inclusive range
    /// let mut string = AntString::new(&input);
    /// string.truncate(1..=5);
    /// assert_eq!(string.to_string(), "12345".to_string());
    ///
    /// // Range to
    /// let mut string = AntString::new(&input);
    /// string.truncate(..5);
    /// assert_eq!(string.to_string(), "01234".to_string());
    /// ```
    ///
    /// # Panics
    ///
    /// * Just like a regular string slice, using a range outside of the
    ///   actual size of the [`AntString`] will panic.
    pub fn truncate(&mut self, range: impl FromRange) {
        let len = self.len();
        let (start, end) = range.into_start_end(len);
        assert!(len >= end, "byte index: {} is out of bounds of `{}`", len, self);

        // Truncate the end
        let (slice_index, end) = self.index(end);
        self.inner.truncate(slice_index + 1);
        let (annotation, last) = &self.inner[slice_index];
        self.inner[slice_index] = (annotation, &last[..end]);

        // Drain the start
        let (slice_index, start) = self.index(start);
        drop(self.inner.drain(..slice_index));
        let (annotation, first) = &self.inner[0];
        let new_start = &first[start..];

        self.inner[0] = (annotation, new_start);
    }

    /// Remove the last char from the string
    /// ```ignore
    /// use anathema::antstring::AntString;
    /// let input = ["012", "345"];
    /// let mut string = AntString::new(input.as_slice());
    /// let c = string.pop().unwrap();
    /// assert_eq!(c, '5');
    /// assert_eq!(string.to_string(), "01234".to_string());
    /// ```
    #[cfg(test)]
    pub fn pop(&mut self) -> Option<char> {
        if self.is_empty() {
            return None;
        }

        let last_char = self.inner.iter().rev().next().and_then(|(_, s)| s.chars().last())?;
        let remove = last_char.len_utf8();
        let to = self.len() - remove;
        self.truncate(..to);

        Some(last_char)
    }

    // Index is operating on processed slices
    fn index(&self, index: usize) -> (usize, usize) {
        let mut offset = 0;
        for (slice_index, (_, slice)) in self.inner.iter().enumerate() {
            if index - offset > slice.len() {
                offset += slice.len();
                continue;
            }

            return (slice_index, index - offset);
        }

        panic!("byte index {index} is out of bounds of `{self}`");
    }
}

impl<'a, T> AntString<'a, T> {
    /// An iterator over the characters of the inner string slices
    #[must_use]
    pub fn annotated_chars(&'a self) -> AnnotatedChars<'a, impl Iterator<Item = &'a (&'a T, &'a str)>, T> {
        AnnotatedChars::new(self.inner.iter())
    }
}

// -----------------------------------------------------------------------------
//     - Display -
// -----------------------------------------------------------------------------
impl<'a, T> fmt::Display for AntString<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_annoation, slice) in &self.inner {
            write!(f, "{slice}")?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Debug -
// -----------------------------------------------------------------------------
impl<'a, T> fmt::Debug for AntString<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_annotation, slice) in &self.inner {
            write!(f, "{slice:?}")?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Unicode width -
// -----------------------------------------------------------------------------
impl<'a, T> UnicodeWidthStr for AntString<'a, T> {
    fn width(&self) -> usize {
        self.inner.iter().map(|(_, s)| s.width()).sum()
    }

    fn width_cjk(&self) -> usize {
        self.inner.iter().map(|(_, s)| s.width_cjk()).sum()
    }
}

// -----------------------------------------------------------------------------
//     - Annotated chars -
// -----------------------------------------------------------------------------
/// An iterator over annotated characters.
pub struct AnnotatedChars<'a, T, U>
where
    T: Iterator<Item = &'a (&'a U, &'a str)>,
{
    inner: T,
    current: Option<(&'a U, StdChars<'a>)>,
}

impl<'a, T, U> AnnotatedChars<'a, T, U>
where
    T: Iterator<Item = &'a (&'a U, &'a str)>,
{
    fn new(mut inner: T) -> Self {
        let current = inner.next().map(|(annotation, slice)| (*annotation, slice.chars()));
        Self { inner, current }
    }
}

impl<'a, T, U> Iterator for AnnotatedChars<'a, T, U>
where
    T: Iterator<Item = &'a (&'a U, &'a str)>,
{
    type Item = (&'a U, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (annotation, current) = self.current.as_mut()?;
        match current.next() {
            Some(s) => Some((annotation, s)),
            None => {
                self.current = self.inner.next().map(|(annotation, s)| (*annotation, s.chars()));
                let (annotation, current) = self.current.as_mut()?;
                current.next().map(|c| (*annotation, c))
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Bytes -
// -----------------------------------------------------------------------------
/// An iterator over the bytes of the [`AntString`]
pub struct Bytes<'a, T: Iterator<Item = &'a str>> {
    inner: T,
    current: Option<StdBytes<'a>>,
}

#[cfg(test)]
impl<'a, T: Iterator<Item = &'a str>> Bytes<'a, T> {
    fn new(mut inner: T) -> Self {
        let current = inner.next().map(str::bytes);
        Self { inner, current }
    }
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for Bytes<'a, T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_mut()?;
        match current.next() {
            Some(s) => Some(s),
            None => {
                self.current = self.inner.next().map(str::bytes);
                self.current.as_mut()?.next()
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Chars -
// -----------------------------------------------------------------------------
/// An iterator over the chars of the [`AntString`]
pub struct Chars<'a, T: Iterator<Item = &'a str>> {
    inner: T,
    current: Option<StdChars<'a>>,
}

#[cfg(test)]
impl<'a, T: Iterator<Item = &'a str>> Chars<'a, T> {
    fn new(mut inner: T) -> Self {
        let current = inner.next().map(str::chars);
        Self { inner, current }
    }
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for Chars<'a, T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_mut()?;
        match current.next() {
            Some(s) => Some(s),
            None => {
                self.current = self.inner.next().map(str::chars);
                self.current.as_mut()?.next()
            }
        }
    }
}

/// An iterator over the chars of the [`AntString`]
pub struct CharsRev<'a, T: Iterator<Item = &'a str>> {
    inner: T,
    current: Option<Rev<StdChars<'a>>>,
}

impl<'a, T: Iterator<Item = &'a str>> CharsRev<'a, T> {
    fn new(mut inner: T) -> Self {
        let current = inner.next().map(|slice| slice.chars().rev());
        Self { inner, current }
    }
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for CharsRev<'a, T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_mut()?;
        match current.next() {
            Some(s) => Some(s),
            None => {
                self.current = self.inner.next().map(|s| s.chars().rev());
                self.current.as_mut()?.next()
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Char indices -
// -----------------------------------------------------------------------------
/// An iterator over the characters and their index of the [`AntString`]
pub struct CharIndices<'a, T> {
    inner: T,
    current: Option<(usize, StdCharIndices<'a>)>,
    offset: usize,
}

impl<'a, T: Iterator<Item = &'a str>> CharIndices<'a, T> {
    fn new(mut inner: T) -> Self {
        let current = inner.next().map(|slice| (slice.len(), slice.char_indices()));
        Self { inner, current, offset: 0 }
    }
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for CharIndices<'a, T> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (next_offset, current) = self.current.as_mut()?;

        match current.next() {
            Some((i, c)) => Some((i + self.offset, c)),
            None => {
                self.offset += *next_offset;
                self.current = self.inner.next().map(|slice| (slice.len(), slice.char_indices()));
                let (_, ref mut iter) = self.current.as_mut()?;
                iter.next().map(|(i, c)| (i + self.offset, c))
            }
        }
    }
}

// -----------------------------------------------------------------------------
//     - Lines -
// -----------------------------------------------------------------------------
/// Lines iterator for AntString (consumes the annotated string)
pub struct Lines<'a, T> {
    inner: Option<AntString<'a, T>>,
}

impl<'a, T> Lines<'a, T> {
    fn new(inner: AntString<'a, T>) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<'a, T> Iterator for Lines<'a, T> {
    type Item = AntString<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.as_mut()?;
        let pos = match inner.find('\n') {
            Some(pos) => pos,
            None => return self.inner.take(),
        };

        let (mut left, right) = inner.split_at(pos + 1);
        left.remove(left.len() - 1..left.len());
        self.inner = match right.is_empty() {
            false => Some(right),
            true => None,
        };
        Some(left)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn string_and_len()(input in any::<String>())
            (index in 0..input.len() + 1, input in Just(input)) -> (String, usize)
        {
            (input, index)
        }
    }

    #[test]
    fn bytes() {
        let s = [(&(), "a"), (&(), "b")];
        let string = AntString::with_annotations(&s);
        let mut bytes = string.bytes();
        assert_eq!(bytes.next().unwrap(), b'a');
        assert_eq!(bytes.next().unwrap(), b'b');
    }

    proptest! {
        #[test]
        // fn bytes_prop(left in "\\PC*", right in "\\PC*") {
        fn bytes_prop(left in any::<String>(), right in any::<String>()) {
            let string = AntString::new(&[left.as_ref(), right.as_ref()]);

            for (a, b) in left.bytes().chain(right.bytes()).zip(string.bytes()) {
                eprintln!("{a} - {b}");
                prop_assert_eq!(a, b);
            }

        }
    }

    #[test]
    fn chars() {
        let s = [(&(), "a"), (&(), "b")];
        let string = AntString::with_annotations(&s);
        let mut chars = string.chars();
        assert_eq!(chars.next().unwrap(), 'a');
        assert_eq!(chars.next().unwrap(), 'b');
    }

    proptest! {
        #[test]
        fn chars_prop(left in any::<String>(), right in any::<String>()) {
            let string = AntString::new(&[left.as_ref(), right.as_ref()]);

            for (a, b) in left.chars().chain(right.chars()).zip(string.chars()) {
                prop_assert_eq!(a, b);
            }

        }
    }

    #[test]
    fn chars_in_reverse() {
        let s = [(&(), "a"), (&(), "bc")];
        let string = AntString::with_annotations(&s);
        let mut chars = string.chars_rev();
        assert_eq!(chars.next().unwrap(), 'c');
        assert_eq!(chars.next().unwrap(), 'b');
        assert_eq!(chars.next().unwrap(), 'a');
    }

    #[test]
    fn char_indices() {
        let s = [(&(), "a"), (&(), "🍅"), (&(), "b")];
        let string = AntString::with_annotations(&s);
        let mut chars = string.char_indices();
        assert_eq!(chars.next().unwrap(), (0, 'a'));
        assert_eq!(chars.next().unwrap(), (1, '🍅'));
        assert_eq!(chars.next().unwrap(), (5, 'b'));
    }

    proptest! {
        #[test]
        fn char_indices_prop(left in any::<String>()) {
            let string = AntString::new(&[left.as_ref()]);

            for (a, b) in left.char_indices().zip(string.char_indices()) {
                prop_assert_eq!(a, b);
            }

        }
    }

    #[test]
    fn collect() {
        let s = [(&(), "a"), (&(), "b")];
        let string = AntString::with_annotations(&s);
        let chars = string.chars();
        let actual = chars.collect::<String>();
        let expected = "ab".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn split() {
        let s = ["0123", "4", "56"];
        let string = AntString::new(s);
        let (left, right) = string.split_at(3);

        let actual = left.to_string();
        let expected = "012".to_string();
        assert_eq!(expected, actual);

        let actual = right.to_string();
        let expected = "3456".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn split_left() {
        let s = ["0"];
        let string = AntString::new(s);
        let (left, _) = string.split_at(1);
        assert_eq!(left.to_string(), "0".to_string());
    }

    proptest! {
        #[test]
        fn split_left_prop((input, index) in string_and_len()) {
            let index = if index == input.len() + 1 { input.len() } else { index };
            if input.is_char_boundary(index) {
                let string = AntString::new([input.as_ref()]);
                let (left, _) = string.split_at(index);
                let actual = format!("{left}");
                let (expected, _) = input.split_at(index);
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn split_right() {
        let s = ["0"];
        let string = AntString::new(s);
        let (_, right) = string.split_at(0);
        assert_eq!(right.to_string(), "0".to_string());
    }

    #[test]
    fn split_twice() {
        let s = ["012345"];
        let string = AntString::new(s);
        let (_, right) = string.split_at(3);
        let (left, _) = right.split_at(2);
        let actual = format!("{}", left);
        let expected = "34".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn trim_start() {
        let s = ["   ", "  ", "a"];
        let mut string = AntString::new(&s);
        string.trim_start();
        let actual = string.to_string();
        let expected = "a".to_string();
        assert_eq!(expected, actual);

        string.trim_start();
        string.trim_end();
        let actual = format!("{}", string);
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn trim_end() {
        let s = ["a    ", " ", "  "];
        let mut string = AntString::new(&s);
        string.trim_end();
        let actual = format!("{}", string);
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn trim() {
        let s = ["  ", " ", "  a    ", " ", "  "];
        let mut string = AntString::new(&s);
        string.trim();
        let actual = format!("{}", string);
        let expected = "a".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn len() {
        let s = ["12", "3"];
        let mut string = AntString::new(s.as_slice());
        assert_eq!(string.len(), 3);
        string.truncate(1..);
        assert_eq!(string.len(), 2);
        string.truncate(1..);
        assert_eq!(string.len(), 1);
    }

    #[test]
    fn pop() {
        let s = ["0", "1", "2"];
        let mut string = AntString::new(&s);
        let actual = string.pop().unwrap();
        let expected = '2';
        assert_eq!(expected, actual);

        let s = [" ", " "];
        let mut string = AntString::new(&s);
        string.pop();
        let _actual = string.pop().unwrap();
        let _expected = ' ';
        let actual = format!("{string}");
        let expected = String::new();
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove() {
        let mut s = AntString::new(["012|34|56"]);
        s.remove(3..=6);
        let actual = s.to_string();
        let expected = "01256".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn insert() {
        let mut s = AntString::with_annotations([(&1000u32, "012"), (&500, "456")]);
        s.insert(3, "3");
        let actual = s.to_string();
        let expected = "0123456".to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn get() {
        let src = "0123456";
        let string = AntString::new([src]);
        let sub = string.get(1..4);

        let expected = &src[1..4];
        let actual = sub.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn lines() {
        let vanilla_slice = "Goat\nA\nB\n\nC\n";
        let s = AntString::new(["Goat\nA\n", "B\n", "\nC\n"]);
        let lines = s.lines();
        let output = lines.map(|s| s.to_string()).collect::<Vec<_>>();
        let vanilla = vanilla_slice.lines().collect::<Vec<_>>();

        assert_eq!(output, vanilla);
    }

    #[test]
    fn single_line() {
        let vanilla_slice = "No lines here";
        let s = AntString::new(["No li", "nes h", "ere"]);
        let lines = s.lines();
        let output = lines.map(|s| s.to_string()).collect::<Vec<_>>();
        let vanilla = vanilla_slice.lines().collect::<Vec<_>>();

        assert_eq!(output, vanilla);
    }
}
