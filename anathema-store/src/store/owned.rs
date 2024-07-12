use std::cell::RefCell;

use super::shared::SharedKey;
use super::Slab;

// -----------------------------------------------------------------------------
//   - Key -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OwnedKey(u32);

impl From<usize> for OwnedKey {
    fn from(i: usize) -> Self {
        // SAFETY
        // this isn't technically unsafe, and it's unlikely that more than
        // four gigabyte of values would be stored
        Self(i as u32)
    }
}

impl From<OwnedKey> for usize {
    fn from(key: OwnedKey) -> usize {
        key.0 as usize
    }
}

// -----------------------------------------------------------------------------
//   - Storage entity -
// -----------------------------------------------------------------------------
/// Represents a value in the storage.
/// The value can be "checked out" either shared or unique.
#[derive(Debug, Clone)]
pub enum OwnedEntry<T> {
    /// An entry in the store
    Occupied(T),
    /// The value is currently borrowed (unique / writable)
    Unique,
    /// The value is currently borrowed (shared)
    Shared(SharedKey),
}

impl<T> OwnedEntry<T> {
    fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied(_))
    }
}

// -----------------------------------------------------------------------------
//   - Storage -
// -----------------------------------------------------------------------------
pub struct Owned<T> {
    inner: RefCell<Slab<OwnedKey, OwnedEntry<T>>>,
}

impl<T> Owned<T> {
    pub const fn empty() -> Self {
        Self {
            inner: RefCell::new(Slab::empty()),
        }
    }

    pub fn get_shared_key(&self, key: OwnedKey) -> Option<SharedKey> {
        match self.inner.borrow().get(key)? {
            OwnedEntry::Shared(key) => Some(*key),
            _ => None,
        }
    }

    pub fn push(&self, value: T) -> OwnedKey {
        self.inner.borrow_mut().insert(OwnedEntry::Occupied(value))
    }

    pub fn try_set_as_shared(&self, owned_key: OwnedKey, shared_key: SharedKey) -> bool {
        let entry = self
            .inner
            .borrow_mut()
            .try_replace(owned_key, OwnedEntry::Shared(shared_key));

        matches!(entry, Some(OwnedEntry::Unique))
    }

    /// Call closure on a value.
    /// This assumes the value exists and is currently
    /// not shared.
    ///
    /// # Panics
    ///
    /// This will panic if the value does not exist or is
    /// currently shared.
    pub fn with<F, U>(&self, key: OwnedKey, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        let inner = self.inner.borrow();
        inner.get(key).map(|val| match val {
            OwnedEntry::Occupied(val) => f(val),
            OwnedEntry::Unique => panic!("value is already checked out"),
            OwnedEntry::Shared(_) => panic!("value is currently shared"),
        })
    }

    /// Get unique access to the value at a given key.
    ///
    /// # Panics
    ///
    /// Will panic if the value is currently shared, or was removed
    pub fn unique(&self, key: OwnedKey) -> T {
        match self.try_unique(key) {
            Some(value) => value,
            None => panic!("value unavailable"),
        }
    }

    /// Try to get unique access to a value.
    /// The value may or may not exist.
    ///
    /// # Panics
    ///
    /// If the value exists but is in an invalid state
    /// this function will panic.
    pub fn try_unique(&self, key: OwnedKey) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        let output = inner.try_replace(key, OwnedEntry::Unique)?;
        match output {
            OwnedEntry::Occupied(value) => Some(value),
            OwnedEntry::Unique => panic!("value is already checked out"),
            OwnedEntry::Shared(_) => panic!("value is currently shared: {key:?}"),
        }
    }

    /// Remove the value from the storage
    pub fn remove(&self, key: OwnedKey) -> T {
        match self.inner.borrow_mut().remove(key) {
            OwnedEntry::Occupied(value) => value,
            OwnedEntry::Unique => panic!("invalid state (U)"),
            OwnedEntry::Shared(_) => panic!("invalid state"),
        }
    }

    // Return a value to the storage.
    // The value can be either a
    // * Unique borrow
    // * The end of a shared borrow
    pub fn return_unique_borrow(&self, key: OwnedKey, value: T) {
        let val = self.inner.borrow_mut().replace(key, OwnedEntry::Occupied(value));
        match val {
            OwnedEntry::Unique => (),
            OwnedEntry::Shared(_) => (),
            _ => panic!("invalid state"),
        }
    }

    /// This function should only be used for tests
    #[doc(hidden)]
    pub fn count_occupied(&self) -> usize {
        self.inner.borrow().iter_values().filter(|e| e.is_occupied()).count()
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(OwnedKey, &OwnedEntry<T>),
    {
        self.inner.borrow().iter().for_each(|(k, v)| f(k, v));
    }
}

impl<T: std::fmt::Debug> Owned<T> {
    pub fn dump_state(&self) -> String {
        self.inner.borrow().dump_state()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let owned = Owned::empty();
        let key = owned.push(Box::new(123u32));
        let unique = owned.unique(key);
        let value: u32 = *unique;
        assert_eq!(value, 123);
    }

    #[test]
    #[should_panic(expected = "value is already checked out")]
    fn unique_borrow() {
        let owned = Owned::empty();
        let key = owned.push(Box::new(123u32));
        let _ = owned.unique(key);
        let _ = owned.unique(key);
    }

    #[test]
    fn return_unique_borrow() {
        let owned = Owned::empty();
        let key = owned.push(Box::new(123u32));
        let value = owned.unique(key);
        owned.return_unique_borrow(key, value);
        let _value = owned.unique(key);
    }

    #[test]
    #[should_panic(expected = "value unavailable")]
    fn remove() {
        let owned = Owned::empty();
        let key = owned.push(Box::new(123u32));
        let _ = owned.remove(key);
        let _value = owned.unique(key);
    }
}
