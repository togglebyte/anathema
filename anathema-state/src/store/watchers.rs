use std::cell::RefCell;
use std::num::NonZeroU16;

use anathema_store::slab::Slab;
use anathema_store::store::{Monitor, OwnedKey};

use super::WATCHERS;

/// There can be at most `u16::MAX` watchers at any given time, even though there can be up to
/// `u32::MAX` values.
///
/// Watchers should only be used to monitor values for testing.
/// For reactive values use the subscribers instead.
///
/// Once a value changes the watcher is removed, and the value has to be re-watched again.
pub struct Watchers {
    inner: Slab<Monitor, Watcher>,
}

impl Watchers {
    pub const fn new() -> Self {
        Self { inner: Slab::empty() }
    }

    pub(crate) fn insert(&mut self, watcher: Watcher) -> Monitor {
        self.inner.insert(watcher)
    }

    pub(crate) fn remove(&mut self, key: Monitor) -> Watcher {
        self.inner.remove(key)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Watcher(usize);

impl Watcher {
    pub fn new(val: usize) -> Self {
        Self(val)
    }
}

pub(crate) fn monitor(key: &mut OwnedKey, watcher: Watcher) {
    let monitor = WATCHERS.with_borrow_mut(|watchers| watchers.insert(watcher));
    key.set_aux(monitor);
}

pub(super) fn remove_monitor(monitor: Monitor) -> Watcher {
    WATCHERS.with_borrow_mut(|watchers| watchers.remove(monitor))
}
