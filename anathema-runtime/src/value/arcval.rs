//! Multi threaded reactive values
//!
//! These values could cause deadlocks so do mind when using them.
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LazyLock};

use anathema_store::slab::Key;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::State;

type ArcRw<T> = Arc<RwLock<T>>;

static CHANGES: LazyLock<Arc<RwLock<Changes>>> = LazyLock::new(Default::default);
static SUBS: LazyLock<Subs> = LazyLock::new(Default::default);

/// Drain the current changes into a local value.
pub fn drain_changes(local_changes: &mut Changes) {
    let mut changes = CHANGES.write();
    changes.drain_into(local_changes);
}

/// Clear all changes
pub fn clear_all_changes() {
    CHANGES.write().clear();
}

fn changes() -> ArcRw<Changes> {
    CHANGES.clone()
}

fn subs() -> Subs {
    SUBS.clone()
}

#[derive(Debug, Clone, Default)]
pub struct Subs(ArcRw<Vec<Key>>);

impl Subs {
    fn subscribe(&self, key: Key) {
        self.0.write().push(key);
    }

    fn unsubscribe(&self, key: Key) {
        self.0.write().retain(|id| !key.eq(id));
    }
}

/// A value that reacts to change.
///
/// ```
/// # use anathema_state::*;
/// let mut value = Value::<usize>::new(1);
/// *value.to_mut() += 1;
/// ```
pub struct Value<T> {
    inner: ArcRw<T>,
    subs: Subs,
}

impl<T> Value<T>
where
    T: State,
{
    pub fn new(inner: T) -> Self {
        Self { inner, subs: panic!() }
    }

    pub fn anon(&self) -> AnonValue {
        AnonValue {
            inner: self.inner.clone(),
            subs: self.subs.clone(),
        }
    }

    pub fn to_ref(&self) -> ValueRef<'_, T> {
        ValueRef { val: self.inner.read() }
    }

    pub fn to_mut(&mut self) -> ValueMut<'_, T> {
        ValueMut {
            val: self.inner.write(),
        }
    }
}

/// A value reference of a `Value<T>`.
#[derive(Debug)]
pub struct ValueRef<'a, T> {
    val: RwLockReadGuard<'a, T>,
}

impl<T> Deref for ValueRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

pub struct ValueMut<'a, T> {
    val: RwLockWriteGuard<'a, T>,
}

impl<T> Deref for ValueMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

impl<T> DerefMut for ValueMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        panic!();
        // self.changes.inner.push(Change::Changed);
        &mut *self.val
    }
}

/// Anonymous value
pub struct AnonValue {
    inner: ArcRw<dyn State>,
    subs: Subs,
}

impl AnonValue {
    pub(crate) fn subscribe(&self, key: Key) {
        self.subs.subscribe(key);
    }

    pub(crate) fn unsub(&self, key: Key) {
        self.subs.unsubscribe(key);
    }
}
