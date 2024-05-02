use std::cell::RefCell;

use super::{OwnedKey, RcSlab};
use crate::slab::Element;

// -----------------------------------------------------------------------------
//   - Shared key -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SharedKey(usize, OwnedKey);

impl From<SharedKey> for usize {
    fn from(key: SharedKey) -> usize {
        key.0
    }
}

impl From<SharedKey> for OwnedKey {
    fn from(key: SharedKey) -> OwnedKey {
        key.1
    }
}

// -----------------------------------------------------------------------------
//   - Shared storage -
// -----------------------------------------------------------------------------
pub struct Shared<T> {
    inner: RefCell<RcSlab<usize, T>>,
}

impl<T> Shared<T> {
    pub const fn empty() -> Self {
        Self {
            inner: RefCell::new(RcSlab::empty()),
        }
    }

    // Get a shared value under the assumption that the value exists.
    // This should only be called if the Rc::strong count is greater than one
    pub fn get(&self, key: SharedKey) -> Element<T> {
        self.inner
            .borrow_mut()
            .get(key.into())
            .expect("the value exists because the shared key exists")
    }

    pub fn insert(&self, owned_key: OwnedKey, value: T) -> SharedKey {
        let key = self.inner.borrow_mut().insert(value);
        SharedKey(key, owned_key)
    }

    pub fn try_evict(&self, key: SharedKey) -> Option<T> {
        self.inner.borrow_mut().try_remove(key.into())
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(usize, &T),
    {
        self.inner.borrow().iter().for_each(|(k, v)| f(k, v));
    }
}

impl<T: std::fmt::Debug> Shared<T> {
    pub fn dump_state(&self) -> String {
        self.inner.borrow().dump_state()
    }
}
