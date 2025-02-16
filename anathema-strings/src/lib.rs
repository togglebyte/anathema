// NOTE: On a second go-around we can consider removing the buffer and pre-compute the required
//       space with some exceptions

use region::Region;
use storage::Storage;
pub use storage::Transaction;

mod region;
mod storage;

static BUCKET_SIZE: usize = 128;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StrIndex {
    index: u32,
    len: u32,
}

impl StrIndex {
    fn new(index: usize, len: usize) -> Self {
        Self {
            index: index as u32,
            len: len as u32,
        }
    }

    fn to_region(self) -> Region {
        let padding = self.len % BUCKET_SIZE as u32;
        Region {
            start: self.index,
            len: self.len + padding,
        }
    }
}

impl From<(u32, u32)> for StrIndex {
    fn from((index, len): (u32, u32)) -> Self {
        Self { index, len }
    }
}

pub struct HStrings<'slice> {
    inner: Storage<'slice>,
}

impl<'slice> HStrings<'slice> {
    pub fn empty() -> Self {
        Self {
            inner: Storage::empty(),
        }
    }

    pub fn insert_with<F>(&mut self, f: F) -> StrIndex
    where
        F: FnOnce(&mut Transaction<'_, 'slice>),
    {
        let mut tx = self.inner.begin_insert();
        f(&mut tx);
        tx.commit()
    }

    pub fn get(&self, hstr: StrIndex) -> HString<impl Iterator<Item = &str> + Clone> {
        let iter = self.inner.get(hstr);
        HString { inner: iter }
    }

    pub fn remove(&mut self, hstr: StrIndex) {
        self.inner.remove(hstr);
    }
}

pub struct HString<I> {
    inner: I,
}

impl<'hstr, I> Iterator for HString<I>
where
    I: Iterator<Item = &'hstr str>,
{
    type Item = &'hstr str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'hstr, I> std::fmt::Debug for HString<I>
where
    I: Iterator<Item = &'hstr str>,
    I: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let iter = self.inner.clone();
        for s in iter {
            s.fmt(f)?;
        }
        Ok(())
    }
}

impl<'hstr, I> std::fmt::Display for HString<I>
where
    I: Iterator<Item = &'hstr str>,
    I: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let iter = self.inner.clone();
        for s in iter {
            s.fmt(f)?;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//   - Equality -
// -----------------------------------------------------------------------------

impl<'hstr, A, B> PartialEq<HString<B>> for HString<A>
where
    A: Iterator<Item = &'hstr str>,
    A: Clone,
    B: Iterator<Item = &'hstr str>,
    B: Clone,
{
    fn eq(&self, other: &HString<B>) -> bool {
        let mut lhs = self.inner.clone();
        let mut rhs = other.inner.clone();

        loop {
            let a = lhs.next();
            let b = rhs.next();
            if a != b {
                return false;
            }

            if a.is_none() && b.is_none() {
                break true;
            }
        }
    }
}

impl<'hstr, I> PartialEq<str> for HString<I>
where
    I: Iterator<Item = &'hstr str>,
    I: Clone,
{
    fn eq(&self, mut other: &str) -> bool {
        let iter = self.inner.clone();

        for s in iter {
            if s != &other[..s.len()] {
                return false;
            }
            other = &other[s.len()..];
        }
        other.is_empty()
    }
}

impl<'hstr, I> PartialEq<&str> for HString<I>
where
    I: Iterator<Item = &'hstr str>,
    I: Clone,
{
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl<'hstr, I> PartialEq<HString<I>> for &str
where
    I: Iterator<Item = &'hstr str>,
    I: Clone,
{
    fn eq(&self, other: &HString<I>) -> bool {
        other.eq(self)
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Write;

    use super::*;

    #[test]
    fn basic() {
        let mut strings = HStrings::empty();
        let hstr = strings.insert_with(|tx| {
            tx.add_slice("hello");
            tx.add_slice(" ");
            tx.add_slice("world");
        });
        let s = strings.get(hstr);

        assert_eq!("hello world", s);
    }

    #[test]
    fn write_borrowed_and_owned() {
        let mut strings = HStrings::empty();
        let hstr = strings.insert_with(|tx| {
            tx.add_slice("hello");
            write!(tx, " ");
            tx.add_slice("world");
        });
        let s = strings.get(hstr);

        assert_eq!(s, "hello world");
        assert_eq!("hello world", s);
    }

    #[test]
    fn empty_str() {
        let mut strings = HStrings::empty();
        let hstr = strings.insert_with(|_| {});
        let s = strings.get(hstr);
        assert!("hello world" != s);
    }
}
