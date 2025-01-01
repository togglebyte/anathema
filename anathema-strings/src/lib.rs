// NOTE: On a second go-around we can consider removing the buffer and pre-compute the required
//       space with some exceptions

use region::Region;
use storage::{Transaction, Storage};

mod region;
mod storage;

static BUCKET_SIZE: usize = 128;

pub struct Hoppstr {
    index: usize,
    len: usize,
}

impl Hoppstr {
    fn new(index: usize, len: usize) -> Self {
        Self { index, len }
    }

    fn to_region(self) -> Region {
        let padding = self.len % BUCKET_SIZE;
        Region {
            start: self.index,
            len: self.len + padding,
        }
    }
}

pub struct Strings<'slice> {
    inner: Storage<'slice>,
}

impl<'slice> Strings<'slice> {
    pub fn empty() -> Self {
        Self {
            inner: Storage::empty(),
        }
    }

    pub fn insert_with<F>(&mut self, mut f: F) -> Hoppstr
    where
        F: FnMut(&mut Transaction),
    {
        let mut tx = self.inner.begin_insert();
        f(&mut tx);
        tx.commit()
    }

    pub fn get(&self, hstr: Hoppstr) -> HString<impl Iterator<Item = &str> + Clone> {
        let iter = self.inner.get(hstr);
        HString { inner: iter }
    }

    pub fn remove(&mut self, hstr: Hoppstr) {
        self.inner.remove(hstr);
    }
}

pub struct HString<I> {
    inner: I,
}

impl<'hstr, I> HString<I> where I: Iterator<Item = &'hstr str> {}

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
    use super::*;

    #[test]
    fn basic() {
        let mut strings = Strings::empty();
        let hstr = strings.insert_with(|tx| {
            tx.add_slice("hello");
            tx.add_slice(" ");
            tx.add_slice("world");
        });
        let s = strings.get(hstr);

        assert_eq!("hello world", s);
    }

    #[test]
    fn empty_str() {
        let mut strings = Strings::empty();
        let hstr = strings.insert_with(|_| {});
        let s = strings.get(hstr);
        assert!("hello world" != s);
    }
}
