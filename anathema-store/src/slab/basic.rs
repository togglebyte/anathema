use std::ops::{Index, IndexMut};

use crate::slab::Slab;

#[macro_export]
macro_rules! basic_key {
    ($v:vis $name:ident ($num:ty)) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        $v struct $name($num);

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value as $num)
            }
        }

        impl From<$name> for usize {
            fn from(value: $name) -> usize {
                value.0 as usize
            }
        }

    };
}

// -----------------------------------------------------------------------------
//   - Entry -
// -----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone)]
enum Entry<I, T> {
    Vacant(Option<I>),
    Occupied(T),
}

impl<I, T> Entry<I, T> {
    // Insert an Occupied entry in place of a vacant one.
    fn swap(&mut self, value: T) {
        debug_assert!(matches!(self, Entry::Vacant(_)));
        *self = Entry::Occupied(value);
    }

    // Create a new occupied entry
    fn occupied(value: T) -> Self {
        Self::Occupied(value)
    }

    // Will panic if the entry is vacant.
    // An entry should never be vacant where this call is involved.
    //
    // This means this method should never be used outside of update calls.
    pub fn as_occupied_mut(&mut self) -> &mut T {
        match self {
            Entry::Occupied(value) => value,
            Entry::Vacant(_) => unreachable!("invalid state"),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Slab -
// -----------------------------------------------------------------------------
/// A basic slab
#[derive(Debug, Clone, PartialEq)]
pub struct Basic<K, V> {
    next_id: Option<K>,
    inner: Vec<Entry<K, V>>,
}

impl<K, V> Default for Basic<K, V> {
    fn default() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }
}

impl<K, V> Slab for Basic<K, V>
where
    K: Copy,
    K: From<usize>,
    K: PartialEq,
    usize: From<K>,
{
    type Key = K;
    type Value = V;

    // If there is a `self.next_id` then `take` the id (making it None)
    // and replace the vacant entry at the given index.
    //
    // Write the vacant entry's `next_id` into self.next_id, and
    // finally replace the vacant entry with the occupied value
    fn insert(&mut self, value: Self::Value) -> Self::Key {
        match self.next_id.take() {
            Some(index) => {
                let entry = &mut self.inner[usize::from(index)];

                let Entry::Vacant(new_next_id) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value);
                index
            }
            None => {
                self.inner.push(Entry::occupied(value));
                let index = self.inner.len() - 1;
                // assert!(index <= K::MAX, "index exceeds the capacity of the slab");
                K::from(index)
            }
        }
    }

    fn remove(&mut self, key: K) -> Self::Value {
        let mut entry = Entry::Vacant(self.next_id.take());
        self.next_id = Some(key);
        std::mem::swap(&mut self.inner[usize::from(key)], &mut entry);

        match entry {
            Entry::Occupied(val) => val,
            Entry::Vacant(_) => panic!("removal of vacant entry"),
        }
    }

    /// Removes a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied
    fn remove_if<F>(&mut self, key: K, f: F) -> Option<Self::Value>
    where
        F: Fn(&Self::Value) -> bool,
    {
        let old = self.inner.get_mut(usize::from(key))?;

        match old {
            Entry::Occupied(val) => {
                if !f(val) {
                    return None;
                }

                let mut entry = Entry::Vacant(self.next_id.take());
                std::mem::swap(old, &mut entry);
                self.next_id = Some(key);
                let Entry::Occupied(val) = entry else { unreachable!() };
                Some(val)
            }
            Entry::Vacant(_) => None,
        }
    }

    fn get(&self, key: K) -> Option<&Self::Value> {
        match self.inner.get(usize::from(key))? {
            Entry::Occupied(val) => Some(val),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: K) -> Option<&mut Self::Value> {
        match self.inner.get_mut(usize::from(key))? {
            Entry::Occupied(val) => Some(val),
            _ => None,
        }
    }

    fn contains_key(&mut self, key: Self::Key) -> bool {
        let index = usize::from(key);
        if index >= self.inner.len() {
            return false;
        }

        match &self.inner[index] {
            Entry::Vacant(_) => false,
            Entry::Occupied(_) => true,
        }
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)> {
        self.inner.iter().enumerate().filter_map(|(index, entry)| match entry {
            Entry::Occupied(val) => Some((Self::Key::from(index), val)),
            Entry::Vacant(_) => None,
        })
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)> {
        self.inner
            .iter_mut()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                Entry::Occupied(val) => Some((Self::Key::from(index), val)),
                Entry::Vacant(_) => None,
            })
    }

    fn iter_keys(&self) -> impl Iterator<Item = Self::Key> {
        self.inner.iter().enumerate().filter_map(|(index, entry)| match entry {
            Entry::Occupied(_) => Some(Self::Key::from(index)),
            Entry::Vacant(_) => None,
        })
    }

    fn iter_values(&self) -> impl Iterator<Item = &Self::Value> {
        self.inner.iter().filter_map(|entry| match entry {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) => None,
        })
    }

    fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut Self::Value> {
        self.inner.iter_mut().filter_map(|entry| match entry {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) => None,
        })
    }
}

impl<K, V> Basic<K, V>
where
    K: Copy,
    K: From<usize>,
    usize: From<K>,
{
    /// Create an empty slab
    pub const fn empty() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }

    /// Get the next id.
    ///
    /// # Warning
    ///
    /// There is no guarantee that this value will be the same
    /// value produced when doing an insert if another insert has happened
    /// since this value was returned.
    pub fn next_id(&self) -> K {
        match self.next_id {
            Some(id) => id,
            None => K::from(self.inner.len()),
        }
    }

    /// Removes a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied
    pub fn try_remove(&mut self, index: K) -> Option<V> {
        let old = self.inner.get_mut(usize::from(index))?;

        match old {
            Entry::Occupied(_) => {
                let mut entry = Entry::Vacant(self.next_id.take());
                std::mem::swap(old, &mut entry);
                self.next_id = Some(index);
                let Entry::Occupied(val) = entry else { unreachable!() };
                Some(val)
            }
            Entry::Vacant(_) => None,
        }
    }

    /// Try to replace an existing value with a new value.
    /// Unlike [`Self::replace`] this function will not panic
    /// if the value does not exist
    pub fn try_replace(&mut self, index: K, mut new_value: V) -> Option<V> {
        match &mut self.inner[usize::from(index)] {
            Entry::Occupied(value) => {
                std::mem::swap(value, &mut new_value);
                Some(new_value)
            }
            Entry::Vacant(_) => None,
        }
    }

    /// Replace an existing value with a new one.
    ///
    /// # Panics
    ///
    /// Will panic if there is no value at the given index.
    pub fn replace(&mut self, index: K, mut new_value: V) -> V {
        let value = self.inner[usize::from(index)].as_occupied_mut();
        std::mem::swap(value, &mut new_value);
        new_value
    }

    /// # Panics
    ///
    /// Will panic if the value does not exist
    pub fn get_mut_unchecked(&mut self, index: K) -> &mut V {
        match self.inner.get_mut(usize::from(index)) {
            Some(Entry::Occupied(val)) => val,
            _ => panic!("no slot at index {}", usize::from(index)),
        }
    }

    /// Consume all the values in the slab and resets the next id.
    /// This does not replace occupied entries with vacant ones,
    /// but rather drain the underlying storage.
    pub fn consume(&mut self) -> impl Iterator<Item = V> + '_ {
        self.next_id = None;
        self.inner.drain(..).filter_map(|e| match e {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) => None,
        })
    }

    /// This is the total length of the underlying storage,
    /// this is not the total number of values in the slab.
    pub fn total_len(&self) -> usize {
        self.inner.len()
    }

    /// Clear the inner storage and reset the next free slot
    pub fn clear(&mut self) {
        self.inner.clear();
        _ = self.next_id.take();
    }
}

impl<K, V> Index<K> for Basic<K, V>
where
    usize: From<K>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        let entry = &self.inner[usize::from(index)];
        match entry {
            Entry::Occupied(value) => value,
            Entry::Vacant(_) => panic!("vacant slot"),
        }
    }
}

impl<K, V> IndexMut<K> for Basic<K, V>
where
    usize: From<K>,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        let entry = &mut self.inner[usize::from(index)];
        match entry {
            Entry::Occupied(value) => value,
            Entry::Vacant(_) => panic!("vacant slot"),
        }
    }
}

#[cfg(test)]
impl<I, T> Basic<I, T>
where
    I: Copy,
    I: From<usize>,
    I: Into<usize>,
    T: std::fmt::Debug,
{
    #[doc(hidden)]
    pub fn dump_state(&self) -> String {
        use std::fmt::Write;

        let mut s = String::new();

        for (idx, value) in self.inner.iter().enumerate() {
            let _ = match value {
                Entry::Vacant(next) => {
                    let _ = write!(&mut s, "{idx}: vacant ");
                    match next {
                        Some(i) => writeln!(&mut s, "next id: {}", (*i).into()),
                        None => writeln!(&mut s, "no next id"),
                    }
                }
                Entry::Occupied(value) => writeln!(&mut s, "{idx}: {value:?}"),
            };
        }

        let _ = writeln!(&mut s, "---- next id ----");

        let _ = match self.next_id {
            Some(i) => writeln!(&mut s, "next id: {}", i.into()),
            None => writeln!(&mut s, "no next id"),
        };

        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let mut slab = Basic::<usize, _>::empty();
        let index = slab.insert(123);
        let val = slab.remove(index);
        assert_eq!(val, 123);
    }

    #[test]
    fn take() {
        let mut slab = Basic::<usize, _>::empty();
        let index_1 = slab.insert(1);
        let _ = slab.remove(index_1);
        let index_2 = slab.insert(2);
        assert_eq!(index_1, index_2)
    }

    #[test]
    fn update() {
        let mut slab = Basic::<usize, _>::empty();
        let index_1 = slab.insert("hello world");
        slab.replace(index_1, "updated");
        let s = slab.remove(index_1);
        assert_eq!(s, "updated");
    }

    // #[test]
    // fn insert_at_with_no_prior_allocations() {
    //     let mut slab = Basic::<usize, &str>::empty();
    //     slab.insert_at(1, "hello");
    //     assert_eq!(Some(0), slab.next_id);
    //     assert!(matches!(slab.inner[0], Entry::Vacant(None)));
    //     assert_eq!(slab.inner[1], Entry::Occupied("hello"));
    // }

    // #[test]
    // fn insert_at_with_prior_allocations() {
    //     let mut slab = Basic::<usize, &str>::empty();
    //     slab.insert("a");
    //     slab.insert("b");
    //     slab.insert("c");

    //     // Free order: [1, 2, 0]
    //     slab.remove(0);
    //     slab.remove(2);
    //     slab.remove(1);

    //     assert_eq!(Some(1), slab.next_id);
    //     assert_eq!(Entry::Vacant(Some(2)), slab.inner[1]);
    //     assert_eq!(Entry::Vacant(Some(0)), slab.inner[2]);

    //     // Free order: [2, 0]
    //     slab.insert_at(1, "x");

    //     assert_eq!(Some(2), slab.next_id);
    //     assert_eq!(Entry::Vacant(Some(0)), slab.inner[2]);
    // }
}
