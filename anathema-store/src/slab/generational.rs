use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::ops::Deref;

use super::{Index, Ticket};

/// Create a newtype that wraps a `Key`.
/// This implements the following traits:
/// * Debug
/// * PartialEq
/// * Copy
/// * Clone
/// * From<Key>
#[macro_export]
macro_rules! key {
    ($name:ident) => {
        #[derive(Debug, PartialEq, Copy, Clone)]
        pub struct $name(anathema_store::slab::Key);
        impl anathema_store::slab::SlabKey for $name { }

        impl From<anathema_store::slab::Key> for $name {
            fn from(key: anathema_store::slab::Key) -> Self {
                Self(key)
            }
        }

        impl From<$name> for anathema_store::slab::Key {
            fn from(key: $name) -> Self {
                key.0
            }
        }

        impl From<$name> for anathema_store::slab::Index {
            fn from(key: $name) -> Self {
                key.0.into()
            }
        }
    }
}

/// A generation associated with a key.
/// The generation is used to ensure that the same key can be reused without retaining
/// a reference to old data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
#[repr(transparent)]
pub struct Gen(u16);

impl From<u16> for Gen {
    fn from(val: u16) -> Self {
        Self(val)
    }
}

impl Deref for Gen {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Gen {
    fn from(val: usize) -> Self {
        Self(val as u16)
    }
}

impl Display for Gen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "G:{}", self.0)
    }
}

/// A key is a combination of an index and a generation.
/// To access a value using a key the value at the given index has to have a
/// matching generation.
///
/// Bits 0..32: 32-bit key
/// Bits 32..48 is the 16-bit generation
/// Bits 48..64 is the 16-bit aux storage in the key.
///
/// This is used to attach additional data to the key.
#[derive(Hash, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Key(u32);

impl Key {
    const GEN_BITS: usize = 10;
    const INDEX_BITS: usize = 22;
    /// Max, with the generation set to zero
    pub const MAX: Self = Self::new(u32::MAX, 0);
    /// One (generation is set to zero)
    pub const ONE: Self = Self::new(1, 0);
    /// Zero for both index and generation.
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance of a key
    pub const fn new(index: u32, generation: u16) -> Self {
        let index = index << Self::GEN_BITS >> Self::GEN_BITS;
        let generation = (generation as u32) << Self::INDEX_BITS;
        Self(index | generation)
    }

    pub(super) fn bump(mut self) -> Self {
        let generation = self.generation().wrapping_add(1);
        self.set_gen(generation);
        self
    }

    pub(super) fn set_gen(&mut self, new_gen: u16) {
        let generation = (new_gen as u32) << Self::INDEX_BITS;
        self.0 = self.index() as u32 | generation
    }

    /// The index
    pub const fn index(&self) -> usize {
        (self.0 << Self::GEN_BITS >> Self::GEN_BITS) as usize
    }

    /// Get the key generation
    pub const fn generation(&self) -> Gen {
        Gen((self.0 >> Self::INDEX_BITS) as u16)
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}:{}>", self.index(), self.generation().0)
    }
}

impl From<(usize, usize)> for Key {
    fn from((index, generation): (usize, usize)) -> Self {
        Self::new(index as u32, generation as u16)
    }
}

impl From<(usize, Gen)> for Key {
    fn from((index, generation): (usize, Gen)) -> Self {
        (index, generation.0 as usize).into()
    }
}

impl From<Key> for Index {
    fn from(value: Key) -> Self {
        value.index().into()
    }
}

/// Implement this trait for any type acting like a key
pub trait SlabKey: Into<Key> + From<Key> {}

impl SlabKey for Key {}

// -----------------------------------------------------------------------------
//   - Entry -
// -----------------------------------------------------------------------------
#[derive(PartialEq)]
enum Entry<T> {
    Vacant(Option<Key>),
    Occupied(T, Gen),
    CheckedOut(Key),
}

impl<T> Entry<T> {
    // Insert an Occupied entry in place of a vacant one.
    fn swap(&mut self, value: T, generation: Gen) {
        debug_assert!(matches!(self, Entry::Vacant(_)));
        *self = Entry::Occupied(value, generation);
    }

    // Create a new occupied entry
    fn occupied(value: T, generation: Gen) -> Self {
        Self::Occupied(value, generation)
    }
}

impl<T: Debug> Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacant(next_id) => f.debug_tuple("Vacant").field(next_id).finish(),
            Self::Occupied(value, generation) => f
                .debug_tuple(&format!("Occupied<{generation:?}>"))
                .field(value)
                .finish(),
            Self::CheckedOut(key) => f.debug_tuple("CheckedOut").field(key).finish(),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Slab -
// -----------------------------------------------------------------------------
/// A generational slab.
/// Each value inserted is given a generation.
/// If another value is inserted at the same index it will have a new generation.
/// This prevents stale indices pointing to incorrect values.
#[derive(Debug)]
pub struct GenSlab<K, T> {
    next_id: Option<Key>,
    inner: Vec<Entry<T>>,
    _key: PhantomData<K>,
}

impl<K, T> GenSlab<K, T>
where
    K: SlabKey,
{
    /// Create an empty slab
    pub const fn empty() -> Self {
        Self {
            next_id: None,
            inner: vec![],
            _key: PhantomData,
        }
    }

    /// Reserve capacity, this does not fill the underlying storage
    /// with vacant entries.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            next_id: None,
            inner: Vec::with_capacity(cap),
            _key: PhantomData,
        }
    }

    /// Try to replace an existing value if it exists, with a new one.
    /// This will bump the generation.
    pub fn try_replace(&mut self, key: K, mut new_value: T) -> Option<(K, T)> {
        let key = key.into();
        match &mut self.inner.get_mut(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => {
                key.bump();
                *generation = key.generation();
                std::mem::swap(&mut new_value, val);
                Some((key.into(), new_value))
            }
            _ => None,
        }
    }

    /// Replace an existing value with a new one.
    /// This will bump the generation.
    ///
    /// # Panics
    ///
    /// Panics if the entry does not exist
    pub fn replace(&mut self, key: K, mut new_value: T) -> (K, T) {
        let key = key.into();
        match &mut self.inner[key.index()] {
            Entry::Occupied(val, generation) if key.generation() == *generation => {
                key.bump();
                *generation = key.generation();
                std::mem::swap(&mut new_value, val);
                (key.into(), new_value)
            }
            Entry::Occupied(..) => panic!("entry refers to a different value"),
            Entry::CheckedOut(_) => panic!("entry is checked out"),
            Entry::Vacant(..) => panic!("entry no longer exists"),
        }
    }

    /// Closure over a mutable reference to T
    pub fn with_mut<F, U>(&mut self, key: K, f: F) -> U
    where
        F: FnOnce(&mut T, &mut Self) -> U,
    {
        let mut ticket = self.checkout(key);
        let ret = f(&mut ticket, self);
        self.restore(ticket);
        ret
    }

    pub(crate) fn checkout(&mut self, key: K) -> Ticket<Key, T> {
        let key = key.into();
        let mut entry = Entry::CheckedOut(key.into());
        std::mem::swap(&mut entry, &mut self.inner[key.index()]);

        match entry {
            Entry::Occupied(value, generation) if key.generation() == generation => Ticket { value, key },
            Entry::Occupied(_, generation) => panic!(
                "invalid generation, current: {generation:?} | key: {:?}",
                key.generation()
            ),
            Entry::CheckedOut(_) => panic!("value already checked out"),
            Entry::Vacant(_) => panic!("entry has been removed"),
        }
    }

    pub(crate) fn restore(&mut self, Ticket { value, key }: Ticket<Key, T>) {
        let mut entry = Entry::Occupied(value, key.generation());
        std::mem::swap(&mut entry, &mut self.inner[key.index()]);

        match entry {
            Entry::CheckedOut(checked_key) if key.generation() == checked_key.generation() => (),
            _ => panic!("failed to return checked out value"),
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
            Some(id) => id.into(),
            None => Key::new(self.inner.len() as u32, 0).into(),
        }
    }

    // If there is a `self.next_key` then `take` the key (making it None)
    // and replace the vacant entry at the given key.
    //
    // Write the vacant entry's `next_id` into self.next_id, and
    // finally replace the vacant entry with the occupied value
    /// Insert a value into the slab
    pub fn insert(&mut self, value: T) -> K {
        match self.next_id.take() {
            Some(key) => {
                let entry = &mut self.inner[key.index()];

                let Entry::Vacant(new_next_id) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value, key.generation());

                key.into()
            }
            None => {
                let index = Key::new(self.inner.len() as u32, 0);
                self.inner.push(Entry::occupied(value, index.generation()));
                index.into()
            }
        }
    }

    /// Remove a value from the slab, as long as the index and generation matches
    #[must_use]
    pub fn remove(&mut self, key: K) -> Option<T> {
        let mut key = key.into();
        let mut entry = Entry::Vacant(self.next_id.take());
        // Increment the generation
        std::mem::swap(&mut self.inner[key.index()], &mut entry);

        let ret = match entry {
            Entry::Occupied(val, generation) if generation == key.generation() => val,
            Entry::Vacant(..) | Entry::Occupied(..) | Entry::CheckedOut(_) => return None,
        };

        key = key.bump();
        self.next_id = Some(key);

        Some(ret)
    }

    /// Try to remove a value from the slab, where the index and generation matches
    pub fn try_remove(&mut self, key: K) -> Option<T> {
        let key = key.into();
        if self.inner.len() <= key.index() {
            return None;
        }
        let mut entry = Entry::Vacant(self.next_id.take());
        // Increment the generation
        std::mem::swap(&mut self.inner[key.index()], &mut entry);

        let ret = match entry {
            Entry::Occupied(val, generation) if generation == key.generation() => val,
            Entry::Vacant(..) | Entry::Occupied(..) | Entry::CheckedOut(_) => return None,
        };

        key.bump();
        self.next_id = Some(key);

        Some(ret)
    }

    /// Get a reference to a value in the slab
    pub fn get(&self, key: K) -> Option<&T> {
        let key = key.into();
        match self.inner.get(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => Some(val),
            _ => None,
        }
    }

    /// Get a mutable reference to a value in the slab
    pub fn get_mut(&mut self, key: K) -> Option<&mut T> {
        let key = key.into();
        match self.inner.get_mut(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => Some(val),
            _ => None,
        }
    }

    /// Be aware that this will only ever be as performant as
    /// the underlying vector if all entries are occupied.
    ///
    /// E.g if the only slot occupied is 1,000,000, then this will
    /// iterate over 1,000,000 entries to get there.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.inner.iter().filter_map(|e| match e {
            Entry::Occupied(val, _) => Some(val),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Be aware that this will only ever be as performant as
    /// the underlying vector if all entries are occupied.
    ///
    /// E.g if the only slot occupied is 1,000,000, then this will
    /// iterate over 1,000,000 entries to get there.
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.inner.into_iter().filter_map(|e| match e {
            Entry::Occupied(val, _) => Some(val),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Mutably iterate over the values in the slab.
    /// See [`GenSlab::iter`] for information about performance.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.inner.iter_mut().filter_map(|e| match e {
            Entry::Occupied(val, _) => Some(val),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Iterate over the keys and elements
    pub fn iter_keys(&self) -> impl Iterator<Item = (K, &T)> + '_ {
        self.inner.iter().enumerate().filter_map(|(i, e)| match e {
            Entry::Occupied(val, generation) => Some((Key::from((i, *generation)).into(), val)),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }
}

impl<K, T> GenSlab<K, T>
where
    T: std::fmt::Debug,
    K: SlabKey,
{
    #[doc(hidden)]
    pub fn dump_state(&self) -> String {
        use std::fmt::Write;

        let mut s = String::new();

        for (idx, value) in self.inner.iter().enumerate() {
            let _ = match value {
                Entry::Vacant(key) => {
                    let _ = write!(&mut s, "{idx}: vacant ");
                    match key {
                        Some(key) => writeln!(&mut s, "next key: {key:?}"),
                        None => writeln!(&mut s, "no next id"),
                    }
                }
                Entry::Occupied(value, generation) => {
                    writeln!(&mut s, "{idx}: (gen: {}) | {value:?}", generation.0)
                }
                Entry::CheckedOut(key) => writeln!(&mut s, "[x] {key:?}"),
            };
        }

        let _ = writeln!(&mut s, "---- next id ----");

        let _ = match self.next_id {
            Some(key) => writeln!(&mut s, "next key: {key:?}"),
            None => writeln!(&mut s, "no next id"),
        };

        s
    }
}

// -----------------------------------------------------------------------------
//   - Index -
// -----------------------------------------------------------------------------
impl<K, T> std::ops::Index<K> for GenSlab<K, T>
where
    K: SlabKey,
{
    type Output = T;

    fn index(&self, index: K) -> &Self::Output {
        match self.get(index) {
            Some(val) => val,
            None => panic!("invalid index or generation"),
        }
    }
}

impl<K, T> std::ops::IndexMut<K> for GenSlab<K, T>
where
    K: SlabKey,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(val) => val,
            None => panic!("invalid index or generation"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let mut slab = GenSlab::<Key, _>::empty();
        let index = slab.insert(123);
        let val = slab.remove(index).unwrap();
        assert_eq!(val, 123);
    }

    #[test]
    fn remove() {
        let mut slab = GenSlab::<Key, _>::empty();
        let key_1 = slab.insert(1u32);
        let _ = slab.remove(key_1);
        let key_2 = slab.insert(2);
        assert_eq!(key_1.index(), key_2.index());
        assert!(key_1.generation() != key_2.generation());
    }

    #[test]
    fn replace() {
        let mut slab = GenSlab::<Key, _>::empty();
        let key_1 = slab.insert("hello world");
        let (key_1, _) = slab.replace(key_1, "updated");
        let s = slab.remove(key_1).unwrap();
        assert_eq!(s, "updated");
    }

    #[test]
    fn get_and_get_mut() {
        let mut slab = GenSlab::<Key, _>::empty();
        let key = slab.insert(1);

        let value = slab.get_mut(key).unwrap();
        *value = 2;

        let value = slab.get(key).unwrap();
        assert_eq!(*value, 2);
    }

    #[test]
    fn ticket() {
        let mut slab = GenSlab::<Key, _>::empty();
        let key_1 = slab.insert(1);
        let key_2 = slab.insert(2);

        // Check out two values
        let mut ticket_1 = slab.checkout(key_1);
        let mut ticket_2 = slab.checkout(key_2);

        ticket_1.value += 100;
        ticket_2.value += 200;

        // Restore the values
        slab.restore(ticket_2);
        slab.restore(ticket_1);

        assert_eq!(*slab.get(key_1).unwrap(), 101);
        assert_eq!(*slab.get(key_2).unwrap(), 202);
    }

    #[test]
    #[should_panic(expected = "value already checked out")]
    fn double_checkout() {
        let mut slab = GenSlab::<Key, _>::empty();
        let key_1 = slab.insert(1);
        let _t1 = slab.checkout(key_1);
        let _t2 = slab.checkout(key_1);
    }

    #[test]
    fn bump_test() {
        let index = Key::new(0, 0);
        let index = index.bump();
        let mut index = index.bump();
        index.set_gen(u16::MAX);
        let index = index.bump();

        assert_eq!(index.generation().0, 0);
    }

    #[test]
    fn from_values() {
        let index = 123;
        let generation = 456u16;
        let key = Key::new(index, generation);
        assert_eq!(key.index(), index as usize);
        assert_eq!(key.generation(), Gen(generation));
    }
}
