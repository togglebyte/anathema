use super::AntString;

/// Checks if a [`AntString`] contains either a [`char`] or a slice of chars.
pub trait Contains<P>: super::sealed::Sealed {
    /// Does the string contain the pattern?
    fn contains(&self, pat: P) -> bool;
}

impl<'a, T> Contains<char> for AntString<'a, T> {
    fn contains(&self, pat: char) -> bool {
        self.inner.iter().any(|(_annotation, s)| s.contains(pat))
    }
}

impl<'a, T> Contains<&[char]> for AntString<'a, T> {
    fn contains(&self, pat: &[char]) -> bool {
        self.inner.iter().any(|(_annotation, s)| s.contains(pat))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains_char() {
        let s = ["ab", "cd"];
        let string = AntString::new(s.as_slice());
        assert!(string.contains('a'));
        assert!(string.contains('b'));
        assert!(string.contains('c'));
        assert!(string.contains('d'));

        assert!(!string.contains('e'));

        let (left, _) = string.split_at(3);
        assert!(left.contains('c'));
    }

    #[test]
    fn contains_slice_o_chars() {
        let s = ["ab", "cd"];
        let string = AntString::new(s.as_slice());
        assert!(string.contains(['a', 'x'].as_slice()));
        assert!(!string.contains(['y', 'x'].as_slice()));
    }
}
