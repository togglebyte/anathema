use anathema_store::slab::Slab;
use anathema_store::store::{Monitor, OwnedKey};

use super::{WATCH_QUEUE, WATCHERS};
use crate::store::values::with_owned;

#[derive(Debug)]
pub enum Watched {
    Timeout,
    Triggered,
}

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

pub(crate) fn monitor(key: OwnedKey, watcher: Watcher) {
    let monitor = WATCHERS.with_borrow_mut(|watchers| watchers.insert(watcher));
    with_owned(key, |val| val.monitor = monitor);
}

pub(crate) fn queue_monitor(monitor: &mut Monitor) {
    let watcher = WATCHERS.with_borrow_mut(|watchers| watchers.remove(*monitor));
    *monitor = Monitor::initial();
    WATCH_QUEUE.with_borrow_mut(|queue| queue.push(watcher));
}
