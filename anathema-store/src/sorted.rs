use std::cmp::Ordering;

/// A sorted list of T.
/// The list is sorted, if needed when the list is accessed.
///
/// This means that values can be added, and the list stays unsorted until
/// either `get` or `get_mut` is called.
#[derive(Debug)]
pub struct SortedList<T> {
    dirty: bool,
    inner: Vec<T>,
}

impl<T: Ord> SortedList<T> {
    pub fn empty() -> Self {
        Self {
            dirty: false,
            inner: vec![],
        }
    }

    fn sort(&mut self) {
        if !self.dirty {
            return;
        }

        self.dirty = false;
        self.inner.sort();
    }

    pub fn get(&mut self, index: usize) -> Option<&T> {
        self.sort();
        self.inner.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.sort();
        self.inner.get_mut(index)
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value);
        self.dirty = true;
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn binary_search_by<F>(&self, f: F) -> Option<usize>
    where
        F: FnMut(&T) -> Ordering,
    {
        self.inner.binary_search_by(f).ok()
    }

    /// Note that this function is not guaranteed to return
    /// a sorted result
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sort_on_get() {
        let mut list = SortedList::empty();

        list.push(10);
        list.push(5);
        list.push(0);

        assert_eq!(list.inner[0], 10);
        assert_eq!(*list.get(0).unwrap(), 0);
    }
}
