use std::marker::PhantomData;

use super::{DataStore, SharedKey};
use crate::slab::Key;

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

pub struct Owned<T>(PhantomData<T>);

impl<T> Owned<T>
where
    T: DataStore<usize>,
{
    pub fn insert(value: T) -> Key {
        T::owned_access(|store| store.insert(OwnedEntry::Occupied(value)))
    }

    pub fn get_shared_key(key: Key) -> Option<SharedKey> {
        T::owned_access(|store| match store.get(key)? {
            OwnedEntry::Shared(key) => Some(*key),
            _ => None,
        })
    }

    pub fn try_set_as_shared(owned_key: Key, shared_key: SharedKey) -> bool {
        let entry = T::owned_access(|store| {
            store
                .try_replace(owned_key, OwnedEntry::Shared(shared_key))
                .map(|(_key, value)| value)
        });
        matches!(entry, Some(OwnedEntry::Unique))
    }

    /// Call a closure on a value.
    /// This assumes the value exists and is currently
    /// not shared.
    ///
    /// # Panics
    ///
    /// This will panic if the value does not exist or is
    /// currently shared.
    pub fn with<F, U>(&self, key: Key, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        T::owned_access(|store| {
            store.get(key).map(|val| match val {
                OwnedEntry::Occupied(val) => f(val),
                OwnedEntry::Unique => panic!("value is already checked out"),
                OwnedEntry::Shared(_) => panic!("value is currently shared"),
            })
        })
    }

    /// Get unique access to the value at a given key.
    ///
    /// # Panics
    ///
    /// Will panic if the value is currently shared, or was removed
    pub fn unique(key: Key) -> T {
        match Self::try_unique(key) {
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
    pub fn try_unique(key: Key) -> Option<T> {
        T::owned_access(|store| {
            let output = store.try_replace(key, OwnedEntry::Unique).map(|(_, val)| val)?;
            match output {
                OwnedEntry::Occupied(value) => Some(value),
                OwnedEntry::Unique => panic!("value is already checked out"),
                OwnedEntry::Shared(_) => panic!("value is currently shared: {key:?}"),
            }
        })
    }

    /// Remove the value from the storage
    pub fn remove(key: Key) -> Option<T> {
        T::owned_access(|store| match store.remove(key)? {
            OwnedEntry::Occupied(value) => Some(value),
            OwnedEntry::Unique => panic!("invalid state (U)"),
            OwnedEntry::Shared(_) => panic!("invalid state"),
        })
    }

    // Return a value to the storage.
    // The value can be either a
    // * Unique borrow
    // * The end of a shared borrow
    pub fn return_unique_borrow(key: Key, value: T) {
        let (_, val) = T::owned_access(|store| store.replace(key, OwnedEntry::Occupied(value)));
        match val {
            OwnedEntry::Unique => (),
            OwnedEntry::Shared(_) => (),
            _ => panic!("invalid state"),
        }
    }

    /// This function should only be used for tests
    #[doc(hidden)]
    pub fn count_occupied() -> usize {
        T::owned_access(|store| store.iter().count())
    }

    // NOTE: is this needed? if so then the iter function needs to include the key as well
    // pub fn for_each<F>(&self, mut f: F)
    // where
    //     F: FnMut(OwnedKey, &OwnedEntry<T>),
    // {
    //     T::owned_access(|store| store.iter().for_each(|k| v);
    // }
}
