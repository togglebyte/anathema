use super::AntString;


/// Finds the position of either a [`char`] or a slice of chars.
pub trait Find<P> : crate::sealed::Sealed {
    /// Find the pattern inside the string, starting from the beginning of the string
    fn find(&self, pat: P) -> Option<usize>;
    /// Find the pattern inside the string, starting from the end of the string
    fn rfind(&self, pat: P) -> Option<usize>;
}

impl<'a, T> Find<char> for AntString<'a, T> {
    fn find(&self, pat: char) -> Option<usize> {
        let mut offset = 0;
        for (_annotation, s) in &self.inner {
            match s.find(pat) {
                Some(pos) => return Some(pos + offset),
                None => offset += s.len(),
            }
        }

        None
    }

    fn rfind(&self, pat: char) -> Option<usize> {
        let mut offset = self.len();
        for (_annotation, s) in self.inner.iter().rev() {
            offset -= s.len();
            match s.rfind(pat) {
                Some(pos) => return Some(pos + offset),
                None => continue,
            }
        }

        None
    }
}

impl<'a, T> Find<&[char]> for AntString<'a, T> {
    fn find(&self, pat: &[char]) -> Option<usize> {
        let mut offset = 0;
        for (_, s) in &self.inner {
            match s.find(pat) {
                Some(pos) => return Some(pos + offset),
                None => offset += s.len(),
            }
        }

        None
    }

    fn rfind(&self, pat: &[char]) -> Option<usize> {
        let mut offset = self.len();
        for (_, s) in self.inner.iter().rev() {
            offset -= s.len();
            match s.rfind(pat) {
                Some(pos) => return Some(pos + offset),
                None => continue,
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_by_char() {
        let input = ["ab ", "c e", "fg"];
        let string = AntString::new(input.as_slice());

        // Find first space
        let actual = string.find(' ').unwrap();
        let expected = 2;
        assert_eq!(expected, actual);

        // Find 'e'
        let actual = string.find('e').unwrap();
        let expected = 5;
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_by_slice() {
        let input = ["ab ", "c e", "fg"];
        let string = AntString::new(input.as_slice());

        // Find first space
        let actual = string.find(['|', ' '].as_slice()).unwrap();
        let expected = 2;
        assert_eq!(expected, actual);

        // Find first space
        let actual = string.find(['f', 'g', '%'].as_slice()).unwrap();
        let expected = 6;
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_by_char() {
        let s = ["12", "3$45", "6$7", "89"];
        let mut string = AntString::new(s.as_slice());
        let pos = string.rfind('$').unwrap();
        string.truncate(..=pos);
        let actual = string.to_string();
        let expected = String::from("123$456$");
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_by_slice() {
        let s = ["123$456$789"];
        let mut string = AntString::new(s.as_slice());
        let pos = string.rfind(['|', '$'].as_slice()).unwrap();
        string.truncate(..=pos);
        let actual = string.to_string();
        let expected = String::from("123$456$");
        assert_eq!(expected, actual);
    }

    #[test]
    fn rfind_on_substring() {
        let s = ["01", "23", "4567"];
        let mut string = AntString::new(s.as_slice());
        string.truncate(1..5);
        let pos = string.rfind('3').unwrap();

        string.truncate(..=pos);
        let actual = format!("{string}");
        let expected = String::from("123");
        assert_eq!(expected, actual);
    }
}
