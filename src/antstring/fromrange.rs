use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub trait FromRange {
    fn into_start_end(self, max: usize) -> (usize, usize);
}

impl FromRange for Range<usize> {
    fn into_start_end(self, _max: usize) -> (usize, usize) {
        (self.start, self.end)
    }
}

impl FromRange for RangeFrom<usize> {
    fn into_start_end(self, max: usize) -> (usize, usize) {
        (self.start, max)
    }
}

impl FromRange for RangeFull {
    fn into_start_end(self, max: usize) -> (usize, usize) {
        (0, max)
    }
}

impl FromRange for RangeInclusive<usize> {
    fn into_start_end(self, _max: usize) -> (usize, usize) {
        let (start, end) = self.into_inner();
        (start, end + 1)
    }
}

impl FromRange for RangeTo<usize> {
    fn into_start_end(self, _max: usize) -> (usize, usize) {
        (0, self.end)
    }
}

impl FromRange for RangeToInclusive<usize> {
    fn into_start_end(self, _max: usize) -> (usize, usize) {
        (0, self.end + 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::antstring::AntString;

    fn test_range(range: impl FromRange, expected_output: &str) {
        let s = ["0123", "4", "5"];
        let mut string = AntString::new(s.as_slice());
        string.truncate(range);
        let actual = format!("{string}");
        assert_eq!(actual, expected_output.to_string());
    }

    #[test]
    fn from_range() {
        test_range(0..5, "01234");
        test_range(2..5, "234");
        test_range(0..0, "");
        test_range(0..1, "0");
    }

    #[test]
    fn from_range_from() {
        test_range(0.., "012345");
        test_range(3.., "345");
    }

    #[test]
    fn from_range_full() {
        test_range(.., "012345");
    }

    #[test]
    fn from_range_inclusive() {
        test_range(0..=5, "012345");
        test_range(1..=5, "12345");
    }

    #[test]
    fn from_range_to() {
        test_range(..5, "01234");
        test_range(..3, "012");
    }

    #[test]
    fn from_range_to_inclusive() {
        test_range(..=5, "012345");
        test_range(..=2, "012");
    }
}
