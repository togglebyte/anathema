use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use super::{Index, Ticket};

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
pub struct Key(u64);

impl Key {
    const AUX_BITS: usize = 16;
    const AUX_OFFSET: usize = Self::INDEX_BITS + Self::GEN_BITS;
    const GEN_BITS: usize = 16;
    const INDEX_BITS: usize = 32;
    const INDEX_OFFSET: usize = Self::AUX_BITS + Self::GEN_BITS;
    /// Max, with the generation set to zero
    pub const MAX: Self = Self::new(u32::MAX, 0);
    /// One (generation is set to zero)
    pub const ONE: Self = Self::new(1, 0);
    /// Zero for both index and generation.
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance of a key
    pub const fn new(index: u32, generation: u16) -> Self {
        let index = (index as u64) << Self::INDEX_OFFSET >> Self::INDEX_OFFSET;
        let generation = (generation as u64) << Self::INDEX_BITS;
        Self(index | generation)
    }

    /// Set the upper auxiliary value
    pub fn set_aux(&mut self, aux: u16) {
        let aux = (aux as u64) << (Self::INDEX_BITS + Self::GEN_BITS);
        self.0 = self.0 << Self::GEN_BITS >> Self::GEN_BITS | aux;
    }

    /// Get the auxiliary value
    pub fn aux(&self) -> u16 {
        (self.0 >> Self::AUX_OFFSET) as u16
    }

    pub(super) fn bump(mut self) -> Self {
        let generation = self.generation().wrapping_add(1);
        self.set_gen(generation);
        self
    }

    pub(super) fn set_gen(&mut self, new_gen: u16) {
        let generation = (new_gen as u64) << Self::INDEX_BITS;
        self.0 = (self.0 << Self::INDEX_OFFSET >> Self::INDEX_OFFSET) | generation;
    }

    /// The index
    pub const fn index(&self) -> usize {
        (self.0 << Self::INDEX_OFFSET >> Self::INDEX_OFFSET) as usize
    }

    /// This gets the index of the key, this is used for debugging only
    pub const fn debug_index(&self) -> usize {
        self.index()
    }

    /// Get the key generation
    pub const fn generation(&self) -> Gen {
        Gen((self.0 >> Self::INDEX_BITS) as u16)
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key <{}:{}>", self.index(), self.generation().0)
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
pub struct GenSlab<T> {
    next_id: Option<Key>,
    inner: Vec<Entry<T>>,
}

impl<T> GenSlab<T> {
    /// Create an empty slab
    pub const fn empty() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }
}

impl<T> GenSlab<T> {
    /// Create an empty slab
    pub const fn empty_aux() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }

    /// Reserve capacity, this does not fill the underlying storage
    /// with vacant entries.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            next_id: None,
            inner: Vec::with_capacity(cap),
        }
    }

    /// Try to replace an existing value if it exists, with a new one.
    /// This will bump the generation.
    pub fn try_replace(&mut self, key: Key, mut new_value: T) -> Option<(Key, T)> {
        match &mut self.inner.get_mut(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => {
                key.bump();
                *generation = key.generation();
                std::mem::swap(&mut new_value, val);
                Some((key, new_value))
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
    pub fn replace(&mut self, key: Key, mut new_value: T) -> (Key, T) {
        match &mut self.inner[key.index()] {
            Entry::Occupied(val, generation) if key.generation() == *generation => {
                key.bump();
                *generation = key.generation();
                std::mem::swap(&mut new_value, val);
                (key, new_value)
            }
            Entry::Occupied(..) => panic!("entry refers to a different value"),
            Entry::CheckedOut(_) => panic!("entry is checked out"),
            Entry::Vacant(..) => panic!("entry no longer exists"),
        }
    }

    /// Closure over a mutable reference to T
    pub fn with_mut<F, U>(&mut self, key: Key, f: F) -> U
    where
        F: FnOnce(&mut T, &mut Self) -> U,
    {
        let mut ticket = self.checkout(key);
        let ret = f(&mut ticket, self);
        self.restore(ticket);
        ret
    }

    pub(crate) fn checkout(&mut self, key: Key) -> Ticket<Key, T> {
        let mut entry = Entry::CheckedOut(key);
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
    pub fn next_id(&self) -> Key {
        match self.next_id {
            Some(id) => id,
            None => Key::new(self.inner.len() as u32, 0),
        }
    }

    // If there is a `self.next_key` then `take` the key (making it None)
    // and replace the vacant entry at the given key.
    //
    // Write the vacant entry's `next_id` into self.next_id, and
    // finally replace the vacant entry with the occupied value
    /// Insert a value into the slab
    pub fn insert(&mut self, value: T) -> Key {
        match self.next_id.take() {
            Some(key) => {
                let entry = &mut self.inner[key.index()];

                let Entry::Vacant(new_next_id) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value, key.generation());

                key
            }
            None => {
                let index = Key::new(self.inner.len() as u32, 0);
                self.inner.push(Entry::occupied(value, index.generation()));
                index
            }
        }
    }

    /// Remove a value from the slab, as long as the index and generation matches
    #[must_use]
    pub fn remove(&mut self, mut key: Key) -> Option<T> {
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
    pub fn try_remove(&mut self, key: Key) -> Option<T> {
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
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.inner.get(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => Some(val),
            _ => None,
        }
    }

    /// Get a mutable reference to a value in the slab
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.inner.get_mut(key.index())? {
            Entry::Occupied(val, generation) if key.generation() == *generation => Some(val),
            _ => None,
        }
    }

    /// Remove this function
    //TODO: remove this
    #[deprecated]
    pub fn count_all_entries(&self) -> usize {
        self.inner.len()
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
    pub fn iter_keys(&self) -> impl Iterator<Item = (Key, &T)> + '_ {
        self.inner.iter().enumerate().filter_map(|(i, e)| match e {
            Entry::Occupied(val, generation) => Some(((i, *generation).into(), val)),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }
}

impl<T> GenSlab<T>
where
    T: std::fmt::Debug,
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
impl<T> std::ops::Index<Key> for GenSlab<T> {
    type Output = T;

    fn index(&self, index: Key) -> &Self::Output {
        match self.get(index) {
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
        let mut slab = GenSlab::empty();
        let index = slab.insert(123);
        let val = slab.remove(index).unwrap();
        assert_eq!(val, 123);
    }

    #[test]
    fn remove() {
        let mut slab = GenSlab::empty();
        let key_1 = slab.insert(1u32);
        let _ = slab.remove(key_1);
        let key_2 = slab.insert(2);
        assert_eq!(key_1.index(), key_2.index());
        assert!(key_1.generation() != key_2.generation());
    }

    #[test]
    fn replace() {
        let mut slab = GenSlab::empty();
        let key_1 = slab.insert("hello world");
        let (key_1, _) = slab.replace(key_1, "updated");
        let s = slab.remove(key_1).unwrap();
        assert_eq!(s, "updated");
    }

    #[test]
    fn get_and_get_mut() {
        let mut slab = GenSlab::empty();
        let key = slab.insert(1);

        let value = slab.get_mut(key).unwrap();
        *value = 2;

        let value = slab.get(key).unwrap();
        assert_eq!(*value, 2);
    }

    #[test]
    fn ticket() {
        let mut slab = GenSlab::empty();
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
        let mut slab = GenSlab::empty();
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

    #[test]
    fn write_aux_store() {
        let index = 123;
        let generation = 456u16;
        let mut key = Key::new(index, generation);
        key.set_aux(42);
        assert_eq!(key.index(), index as usize);
        assert_eq!(key.generation(), Gen(generation));
        assert_eq!(key.aux(), 42);
    }
}
