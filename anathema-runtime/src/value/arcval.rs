//! Multi threaded reactive values
//!
//! These values could cause deadlocks so do mind when using them.
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LazyLock};

use anathema_store::slab::Key;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::value::{Change, Changes, ValueIndex};
use crate::{State, Type};

type ArcRw<T> = Arc<RwLock<T>>;

static CHANGES: LazyLock<Arc<RwLock<Changes>>> = LazyLock::new(Default::default);
static SUBS: LazyLock<Subs> = LazyLock::new(Default::default);

/// Drain the current changes into a local value.
pub fn drain_changes(local_changes: &mut Changes) {
    let mut changes = CHANGES.write();
    changes.drain_into(local_changes);
}

pub fn changed_one(change: Change, key: ValueIndex) {
    let mut changes = CHANGES.write();
    changes.push((key, change));
}

pub fn changed_many(change: Change, keys: impl Iterator<Item = ValueIndex>) {
    let mut changes = CHANGES.write();
    keys.for_each(|key| changes.push((key, change)));
}

fn changes() -> ArcRw<Changes> {
    CHANGES.clone()
}

#[derive(Debug, Default)]
pub struct Subs(ArcRw<Vec<ValueIndex>>);

impl PartialEq for Subs {
    fn eq(&self, other: &Self) -> bool {
        self.0.data_ptr() == other.0.data_ptr()
    }
}

impl Clone for Subs {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Subs {
    fn empty() -> Subs {
        Self(Default::default())
    }

    pub(crate) fn changed(&mut self, change: Change) {
        changed_many(change, self.0.write().iter().copied());
    }

    fn subscribe(&self, key: ValueIndex) {
        self.0.write().push(key);
    }

    fn unsubscribe(&self, key: ValueIndex) {
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
#[derive(Debug)]
pub struct Value<T> {
    inner: ArcRw<T>,
    pub(crate) subs: Subs,
}

impl<V> Value<V>
where
    V: State,
{
    pub fn new(inner: V) -> Self {
        Self {
            inner: Arc::new(RwLock::new(inner)),
            subs: Subs::empty(),
        }
    }

    pub fn reference(&self) -> AnonValue {
        AnonValue {
            inner: self.inner.clone(),
            subs: self.subs.clone(),
        }
    }

    /// Replace the underlying value
    pub fn set(&mut self, new_value: V) {
        *self.inner.write() = new_value;
        self.subs.changed(Change::Changed);
    }

    pub fn to_ref(&self) -> ValueRef<'_, V> {
        ValueRef { val: self.inner.read() }
    }

    pub fn to_mut(&mut self) -> ValueMut<'_, V> {
        ValueMut {
            val: self.inner.write(),
        }
    }

    pub(crate) fn untracked_mut(&mut self) -> (&mut Subs, UntrackedMut<'_, V>) {
        (&mut self.subs, UntrackedMut(self.inner.write()))
    }

    pub(crate) fn changed(&mut self, change: Change) {
        self.subs.changed(change);
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

// -----------------------------------------------------------------------------
//   - Untracked mut -
// -----------------------------------------------------------------------------
pub(crate) struct UntrackedMut<'a, V>(RwLockWriteGuard<'a, V>);

impl<V> Deref for UntrackedMut<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> DerefMut for UntrackedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

/// Anonymous value
/// See rcval.rs for a more indepth description
#[derive(Debug, Clone)]
pub struct AnonValue {
    inner: ArcRw<dyn State>,
    subs: Subs,
}

impl PartialEq for AnonValue {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner) && self.subs == other.subs
    }
}

impl AnonValue {
    pub(crate) fn subscribe(&self, key: ValueIndex) {
        self.subs.subscribe(key);
    }

    pub fn unsubscribe(&self, key: ValueIndex) {
        self.subs.unsubscribe(key);
    }

    pub fn as_state(&self) -> RwLockReadGuard<'_, dyn State> {
        self.inner.read()
    }

    pub fn value<T: 'static>(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        let val = self.inner.read(); // as RwLockReadGuard<'_, dyn std::any::Any>;
        RwLockReadGuard::try_map(val, |val| {
            let val = val as &dyn std::any::Any;
            val.downcast_ref::<T>()
        })
        .ok()
    }

    pub fn type_info(&self) -> Type {
        self.as_state().type_info()
    }
}
