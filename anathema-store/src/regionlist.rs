use std::ops::{Range, RangeBounds};

use crate::slab::{Index, SecondaryMap};
use crate::stack::Stack;

#[derive(Debug)]
struct Entry {
    // The offset to the first value
    offset: usize,
    // The last slot a value was inserted into
    last_index: usize,
}

/// A region list is a collection of regions of values associated with a slab key.
/// Each key entry has a pre-determined sized region associated with it.
///
/// ```
/// let mut region_list = RegionList::<32, Key, ()>::empty();
/// region_list.insert(Key::from((12, 0)), ());
/// ```
#[derive(Debug)]
pub struct RegionList<const SIZE: usize, K, V> {
    values: ValueList<V>,
    keys: SecondaryMap<K, Entry>,
    next: usize,
}

impl<const SIZE: usize, K, V> RegionList<SIZE, K, V>
where
    K: Into<Index>,
    K: Copy,
{
    /// Create an empty instance of a change list
    /// ```
    /// let list = ChangeList::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            values: ValueList::empty(),
            keys: SecondaryMap::empty(),
            next: 0,
        }
    }

    /// Insert a value into the changelist
    pub fn insert(&mut self, key: K, value: V) {
        let insert_at = match self.keys.get_mut(key) {
            Some(entry) => {
                entry.last_index += 1;
                entry.last_index + entry.offset
            }
            None => {
                let entry = Entry {
                    offset: self.next,
                    last_index: 0,
                };
                let offset = entry.offset;
                self.next += SIZE;
                self.keys.insert(key, entry);
                offset
            }
        };

        self.values.insert(insert_at, value);
    }

    /// Drain the values for a given key
    ///
    /// # Panics
    ///
    /// Will panic if there is no entry associated with the given key
    pub fn drain(&mut self, key: K) -> Option<impl Iterator<Item = V> + '_> {
        let entry = self.keys.get(key)?;
        let start = entry.offset;
        let end = entry.offset + entry.last_index + 1;
        Some(self.values.drain(start..end))
    }
}

#[derive(Debug)]
struct ValueList<T> {
    inner: Vec<Option<T>>,
}

impl<T> ValueList<T> {
    fn empty() -> Self {
        Self { inner: vec![] }
    }

    fn insert(&mut self, index: usize, value: T) {
        if self.inner.len() <= index {
            self.inner.resize_with(index + 1, || None);
        }
        self.inner[index].replace(value);
    }

    fn drain(&mut self, range: Range<usize>) -> impl Iterator<Item = T> + '_ {
        self.inner[range].iter_mut().filter_map(|v| v.take())
    }
}
