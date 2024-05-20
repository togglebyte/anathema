use std::fmt::{self, Debug};
use std::ops::Deref;

use super::{Index, Ticket};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Gen(u16);

impl Gen {
    fn bump(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

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

/// A key is a combination of an index and a generation.
/// To access a value using a key the value at the given index
/// has to have a matching generation.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    index: Index,
    gen: Gen,
}

impl Key {
    /// Max key value
    pub const MAX: Self = Self {
        index: Index(u32::MAX),
        gen: Gen(u16::MAX),
    };
    /// One
    pub const ONE: Self = Self::new(1);
    /// Zero
    pub const ZERO: Self = Self::new(0);

    /// Create a new key with a generation of zero
    pub const fn new(index: u32) -> Self {
        Self {
            index: Index(index),
            gen: Gen(0),
        }
    }

    fn bump(&mut self) {
        self.gen.bump();
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key <{}:{}>", self.index.0, self.gen.0)
    }
}

impl From<(usize, usize)> for Key {
    fn from((index, gen): (usize, usize)) -> Self {
        Self {
            index: index.into(),
            gen: gen.into(),
        }
    }
}

impl From<Key> for Index {
    fn from(value: Key) -> Self {
        value.index
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
    fn swap(&mut self, value: T, gen: Gen) {
        debug_assert!(matches!(self, Entry::Vacant(_)));
        std::mem::swap(self, &mut Entry::Occupied(value, gen));
    }

    // Create a new occupied entry
    fn occupied(value: T, gen: Gen) -> Self {
        Self::Occupied(value, gen)
    }
}

impl<T: Debug> Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacant(next_id) => f.debug_tuple("Vacant").field(next_id).finish(),
            Self::Occupied(value, gen) => f.debug_tuple(&format!("Occupied<{gen:?}>")).field(value).finish(),
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

    /// Reserve capacity, this does not fill the underlying storage
    /// with vacant entries.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            next_id: None,
            inner: Vec::with_capacity(cap),
        }
    }

    /// Replace an existing value with a new one.
    /// This will bump the generation.
    pub fn replace(&mut self, mut key: Key, mut new_value: T) -> Option<(Key, T)> {
        match &mut self.inner[*key.index as usize] {
            Entry::Occupied(val, gen) if key.gen == *gen => {
                key.bump();
                *gen = key.gen;
                std::mem::swap(&mut new_value, val);
                Some((key, new_value))
            }
            _ => None,
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

    pub(crate) fn checkout(&mut self, key: Key) -> Ticket<T> {
        let mut entry = Entry::CheckedOut(key);
        std::mem::swap(&mut entry, &mut self.inner[*key.index as usize]);

        match entry {
            Entry::Occupied(value, gen) if key.gen == gen => Ticket { value, key },
            Entry::CheckedOut(_) => panic!("value already checked out"),
            _ => panic!("no entry maching the key"),
        }
    }

    pub(crate) fn restore(&mut self, Ticket { value, key }: Ticket<T>) {
        let mut entry = Entry::Occupied(value, key.gen);
        std::mem::swap(&mut entry, &mut self.inner[*key.index as usize]);

        match entry {
            Entry::CheckedOut(checked_key) if key.gen == checked_key.gen => (),
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
            None => Key::new(self.inner.len() as u32),
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
                let entry = &mut self.inner[*key.index as usize];

                let Entry::Vacant(new_next_id) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value, key.gen);

                key
            }
            None => {
                self.inner.push(Entry::occupied(value, Gen(0)));
                Key {
                    index: (self.inner.len() - 1).into(),
                    gen: Gen(0),
                }
            }
        }
    }

    /// Remove a value from the slab, as long as the index and generation matches
    #[must_use]
    pub fn remove(&mut self, mut key: Key) -> Option<T> {
        let mut entry = Entry::Vacant(self.next_id.take());
        // Increment the generation
        std::mem::swap(&mut self.inner[*key.index as usize], &mut entry);

        let ret = match entry {
            Entry::Occupied(val, gen) if gen == key.gen => val,
            Entry::Vacant(..) | Entry::Occupied(..) | Entry::CheckedOut(_) => return None,
        };

        key.bump();
        self.next_id = Some(key);

        Some(ret)
    }

    /// Try to remove a value from the slab, where the index and generation matches
    pub fn try_remove(&mut self, mut key: Key) -> Option<T> {
        if self.inner.len() <= *key.index as usize {
            return None;
        }
        let mut entry = Entry::Vacant(self.next_id.take());
        // Increment the generation
        std::mem::swap(&mut self.inner[*key.index as usize], &mut entry);

        let ret = match entry {
            Entry::Occupied(val, gen) if gen == key.gen => val,
            Entry::Vacant(..) | Entry::Occupied(..) | Entry::CheckedOut(_) => return None,
        };

        key.bump();
        self.next_id = Some(key);

        Some(ret)
    }

    /// Get a reference to a value in the slab
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.inner.get(*key.index as usize)? {
            Entry::Occupied(val, gen) if key.gen.eq(gen) => Some(val),
            _ => None,
        }
    }

    /// Get a mutable reference to a value in the slab
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.inner.get_mut(*key.index as usize)? {
            Entry::Occupied(val, gen) if key.gen.eq(gen) => Some(val),
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
            Entry::Occupied(val, gen) => Some((
                Key {
                    index: i.into(),
                    gen: *gen,
                },
                val,
            )),
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
                        Some(key) => {
                            writeln!(&mut s, "next key: {}:{}", key.index.0, key.gen.0)
                        }
                        None => writeln!(&mut s, "no next id"),
                    }
                }
                Entry::Occupied(value, gen) => {
                    writeln!(&mut s, "{idx}: (gen: {}) | {value:?}", gen.0)
                }
                Entry::CheckedOut(key) => writeln!(&mut s, "[x] {key:?}"),
            };
        }

        let _ = writeln!(&mut s, "---- next id ----");

        let _ = match self.next_id {
            Some(key) => writeln!(&mut s, "next key: {}:{}", *key.index, *key.gen),
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
        let mut slab = GenSlab::empty();
        let index = slab.insert(123);
        let val = slab.remove(index).unwrap();
        assert_eq!(val, 123);
    }

    #[test]
    fn remove() {
        let mut slab = GenSlab::empty();
        let key_1 = slab.insert(1);
        let _ = slab.remove(key_1);
        let key_2 = slab.insert(2);
        assert_eq!(key_1.index, key_2.index);
        assert!(key_1.gen != key_2.gen);
    }

    #[test]
    fn replace() {
        let mut slab = GenSlab::empty();
        let key_1 = slab.insert("hello world");
        let (key_1, _) = slab.replace(key_1, "updated").unwrap();
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
}
