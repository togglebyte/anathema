use std::marker::PhantomData;

use super::{DataStore, Owned, OwnedEntry};
use crate::slab::{Key, SharedSlab};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SharedKey(pub usize, pub Key);

impl From<SharedKey> for usize {
    fn from(key: SharedKey) -> usize {
        key.0
    }
}

impl From<SharedKey> for Key {
    fn from(key: SharedKey) -> Key {
        key.1
    }
}

pub struct Shared<T>(PhantomData<T>);

impl<T> Shared<T>
where
    T: DataStore<usize>,
{
    /// Get a shared value under the assumption that the value exists.
    /// This should only be called if the Rc / Arc strong count is greater than one.
    ///
    /// E.g the value has been borrowed already
    pub fn get(key: SharedKey) -> <<T as DataStore<usize>>::Slab as SharedSlab<usize, T>>::Element {
        T::shared_access(|store| {
            store
                .get(key.into())
                .expect("the value exists because the shared key exists")
        })
    }

    pub fn insert(owned_key: Key, value: T) -> SharedKey {
        T::shared_access(|store| {
            let key = store.insert(value);
            SharedKey(key, owned_key)
        })
    }

    pub fn try_evict(key: SharedKey) -> Option<T> {
        T::shared_access(|store| store.try_remove(key.into()))
    }

    // TODO: if we need this just add for_each directly on the trait
    // pub fn for_each<F>(&self, mut f: F)
    // where
    //     F: FnMut(usize, &T),
    // {
    //     T::shared_access(|store| store.iter().for_each(|(k, v)| f(k, v)));
    // }

    pub(crate) fn try_make_shared(
        owned_key: Key,
    ) -> Option<(
        SharedKey,
        <<T as DataStore<usize>>::Slab as SharedSlab<usize, T>>::Element,
    )> {
        match Owned::<T>::get_shared_key(owned_key) {
            Some(key) => {
                let thing = T::shared_access(|store| store.get(key.into()))?;
                Some((key, thing))
            }
            None => {
                // Transfer value from OWNED to SHARED
                let value = Owned::<T>::try_unique(owned_key)?;
                let key = Self::insert(owned_key, value);
                Owned::<T>::try_set_as_shared(owned_key, key).then(|| {
                    let value = T::shared_access(|store| store.get(key.into()))?; //lookup_shared(key);
                    Some((key, value))
                })?
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use crate::slab::{ArcSlab, GenSlab};

    impl DataStore<usize> for String {
        type Slab = ArcSlab<usize, Self>;

        fn owned_access<F, U>(f: F) -> U
        where
            F: FnOnce(&mut crate::slab::GenSlab<OwnedEntry<Self>>) -> U,
            Self: Sized,
        {
            static OWNED: Mutex<GenSlab<OwnedEntry<String>>> = Mutex::new(GenSlab::empty());
            OWNED.lock().map(|mut owned| f(&mut *owned)).unwrap()
        }

        fn shared_access<F, U>(f: F) -> U
        where
            F: FnOnce(&mut Self::Slab) -> U,
            Self: Sized,
        {
            static SHARED: Mutex<ArcSlab<usize, String>> = Mutex::new(ArcSlab::empty());
            SHARED.lock().map(|mut shared| f(&mut *shared)).unwrap()
        }
    }

    #[test]
    fn borrow_value() {
        let s = String::new();
        let key = Owned::insert(s);
        let mut s: String = Owned::unique(key);
        s.push_str("hello world");
        Owned::return_unique_borrow(key, s);

        let (shared_key, shared1) = Shared::<String>::try_make_shared(key).unwrap();
        let (shared_key, shared2) = Shared::<String>::try_make_shared(key).unwrap();
        eprintln!("{:?}", shared1);
        let mut s: String = Owned::unique(key);
        eprintln!("{}", *shared2);

        panic!();
    }
}
