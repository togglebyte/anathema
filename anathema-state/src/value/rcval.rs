//! Single threaded values
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use anathema_store::slab::Key;

use crate::value::changes::{Change, Changes};
use crate::State;

type RcRefCell<T> = Rc<RefCell<T>>;

thread_local! {
    static CHANGES: RefCell<Changes> = RefCell::new(Default::default());
}

/// Drain the current changes into a local value.
pub fn drain_changes(local_changes: &mut Changes) {
    CHANGES.with_borrow_mut(|changes| changes.drain_into(local_changes));
}

/// Clear all changes
pub fn clear_all_changes() {
    CHANGES.with_borrow_mut(|changes| changes.clear());
}

pub(crate) fn changed(key: Key, change: Change) {
    CHANGES.with_borrow_mut(|changes| changes.push((key, change)));
}

/// Keys that are subscribing to changes of a given value
#[derive(Debug, Default, PartialEq)]
pub struct Subs(RcRefCell<Vec<Key>>);

impl Clone for Subs {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Subs {
    pub(crate) fn changed(&mut self, change: Change) {
        self.0
            .borrow_mut()
            .iter()
            .copied()
            .for_each(|key| changed(key, change));
    }

    fn empty() -> Subs {
        Self(Default::default())
    }

    fn subscribe(&self, key: Key) {
        self.0.borrow_mut().push(key);
    }

    fn unsubscribe(&self, key: Key) {
        self.0.borrow_mut().retain(|id| !key.eq(id));
    }

    fn with_keys<F>(&self, f: F)
    where
        F: Fn(Key),
    {
        self.0.borrow().iter().copied().for_each(f)
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
    inner: RcRefCell<T>,
    pub(crate) subs: Subs,
}

impl<T> Value<T>
where
    T: State,
{
    /// Create a new instance of a value
    pub fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
            subs: Subs::empty(),
        }
    }

    /// Get an anonymous value
    pub fn anon(&self) -> AnonValue {
        let weak = Rc::downgrade(&self.inner);
        AnonValue {
            inner: self.inner.clone(), //weak,
            subs: self.subs.clone(),
        }
    }

    /// Replace the underlying value
    pub(crate) fn set(&mut self, empty: T) {
        _ = self.inner.replace(empty);
        self.subs.changed(Change::Changed);
    }

    /// Mutable access to the underlying value.
    pub fn to_mut(&mut self) -> ValueMut<'_, T> {
        ValueMut {
            val: self.inner.borrow_mut(),
            subs: self.subs.clone(),
        }
    }

    /// Immutable access to the underlying value.
    pub fn to_ref(&self) -> ValueRef<'_, T> {
        ValueRef {
            val: self.inner.borrow(),
        }
    }

    pub(crate) fn untracked_mut(&mut self) -> (&mut Subs, UntrackedMut<'_, T>) {
        (&mut self.subs, UntrackedMut(self.inner.borrow_mut()))
    }

    pub(crate) fn changed(&mut self, change: Change) {
        self.subs.changed(change);
    }
}

/// A value reference of a `Value<T>`.
#[derive(Debug)]
pub struct ValueRef<'a, T> {
    val: Ref<'a, T>,
}

impl<T> Deref for ValueRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

// -----------------------------------------------------------------------------
//   - Value mut -
// -----------------------------------------------------------------------------
/// A mutable reference to a value that registers changes whenever the value is dereferenced
pub struct ValueMut<'a, T> {
    val: RefMut<'a, T>,
    subs: Subs,
}

impl<'a, T> ValueMut<'a, T> {
    pub fn sub(&mut self, sub: Key) {
        self.subs.subscribe(sub);
    }
}

impl<T> Deref for ValueMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

impl<T> DerefMut for ValueMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.subs.changed(Change::Changed);
        &mut *self.val
    }
}

// -----------------------------------------------------------------------------
//   - Untracked mut -
// -----------------------------------------------------------------------------
pub(crate) struct UntrackedMut<'a, T>(RefMut<'a, T>);

impl<T> Deref for UntrackedMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> DerefMut for UntrackedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

/// This is an anonymous value without an associated value type.
///
/// This type serves two purposed:
/// 1. Observe changes to the value.
/// 2. Act as a "maybe" value (the value might be dropped between uses).
///
/// This can never be mutable, just like a shared reference in Rust can not
/// be mutable.
///
/// `SharedValue` exists as a maybe-value, as the owner of the value can drop
/// the value regardless of how many shared values there are.
/// This is why `load<T>()` returns an option.
/// ```
/// # use anathema_state::*;
/// let value = Value::new(123u32);
/// let v1 = value.anon();
/// let v2 = value.anon();
///
/// assert_eq!(*v1.value::<u32>().unwrap(), 123);
/// ```
#[derive(Debug, Clone)]
pub struct AnonValue {
    inner: Rc<RefCell<dyn State>>,
    subs: Subs,
}

impl PartialEq for AnonValue {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner) && self.subs == other.subs
    }
}

impl AnonValue {
    pub(crate) fn subscribe(&self, key: Key) {
        self.subs.subscribe(key);
    }

    pub(crate) fn unsub(&self, key: Key) {
        self.subs.unsubscribe(key);
    }

    pub fn as_state(&self) -> Ref<'_, dyn State> {
        self.inner.borrow()
    }

    pub fn value<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let val = self.inner.borrow() as Ref<'_, dyn std::any::Any>;
        Ref::filter_map(val, |x| x.downcast_ref::<T>()).ok()
    }
}
