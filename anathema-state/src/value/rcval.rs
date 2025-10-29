//! Single threaded values
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use anathema_store::slab::Key;

use crate::value::changes::Change;
use crate::value::{SubKey, Type};
use crate::State;

type RcRefCell<T> = Rc<RefCell<T>>;

// thread_local! {
//     static CHANGES: RefCell<Changes> = RefCell::new(Default::default());
// }

// /// Drain the current changes into a local value.
// pub fn drain_changes(local_changes: &mut Changes) {
//     CHANGES.with_borrow_mut(|changes| changes.drain_into(local_changes));
// }

// /// Clear all changes
// pub fn clear_all_changes() {
//     CHANGES.with_borrow_mut(|changes| changes.clear());
// }

// pub(crate) fn changed(key: (Key, Key), change: Change) {
//     CHANGES.with_borrow_mut(|changes| changes.push((key, change)));
// }

/// Keys that are subscribing to changes of a given value
#[derive(Debug, Default, PartialEq)]
pub struct Subs<K>(RcRefCell<Vec<K>>);

impl<K: Clone> Clone for Subs<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K: SubKey> Subs<K> {
    pub(crate) fn changed(&mut self, change: Change) {
        K::changed_many(change, self.0.borrow_mut().iter().copied());
    }

    fn empty() -> Subs<K> {
        Self(Default::default())
    }

    fn subscribe(&self, key: K) {
        self.0.borrow_mut().push(key);
    }

    fn unsubscribe(&self, key: K) {
        self.0.borrow_mut().retain(|id| !key.eq(id));
    }

    fn with_keys<F>(&self, f: F)
    where
        F: Fn(K),
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
pub struct Value<K, V> {
    inner: RcRefCell<V>,
    pub(crate) subs: Subs<K>,
}

impl<K, V> Value<K, V>
where
    V: State<K>,
    K: SubKey,
{
    /// Create a new instance of a value
    pub fn new(inner: V) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
            subs: Subs::empty(),
        }
    }

    /// Get an anonymous value
    pub fn reference(&self) -> AnonValue<K> {
        let weak = Rc::downgrade(&self.inner);
        AnonValue {
            inner: self.inner.clone(), //weak,
            subs: self.subs.clone(),
        }
    }

    /// Replace the underlying value
    pub fn set(&mut self, new_value: V) {
        _ = self.inner.replace(new_value);
        self.subs.changed(Change::Changed);
    }

    /// Mutable access to the underlying value.
    pub fn to_mut(&mut self) -> ValueMut<'_, K, V> {
        ValueMut {
            val: self.inner.borrow_mut(),
            subs: self.subs.clone(),
        }
    }

    /// Immutable access to the underlying value.
    pub fn to_ref(&self) -> ValueRef<'_, V> {
        ValueRef {
            val: self.inner.borrow(),
        }
    }

    pub(crate) fn untracked_mut(&mut self) -> (&mut Subs<K>, UntrackedMut<'_, V>) {
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
pub struct ValueMut<'a, K, V> {
    val: RefMut<'a, V>,
    subs: Subs<K>,
}

impl<'a, K: SubKey, V> ValueMut<'a, K, V> {
    pub fn sub(&mut self, sub: K) {
        self.subs.subscribe(sub);
    }
}

impl<K, V> Deref for ValueMut<'_, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &*self.val
    }
}

impl<K: SubKey, V> DerefMut for ValueMut<'_, K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.subs.changed(Change::Changed);
        &mut *self.val
    }
}

// -----------------------------------------------------------------------------
//   - Untracked mut -
// -----------------------------------------------------------------------------
pub(crate) struct UntrackedMut<'a, V>(RefMut<'a, V>);

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
pub struct AnonValue<K: SubKey> {
    inner: Rc<RefCell<dyn State<K>>>,
    subs: Subs<K>,
}

impl<K: SubKey> PartialEq for AnonValue<K> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner) && self.subs == other.subs
    }
}

impl<K: SubKey> AnonValue<K> {
    pub fn subscribe(&self, key: impl Into<K>) {
        self.subs.subscribe(key.into());
    }

    pub fn unsubscribe(&self, key: impl Into<K>) {
        self.subs.unsubscribe(key.into());
    }

    pub fn as_state(&self) -> Ref<'_, dyn State<K>> {
        self.inner.borrow()
    }

    pub fn value<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let val = self.inner.borrow() as Ref<'_, dyn std::any::Any>;
        Ref::filter_map(val, |x| x.downcast_ref::<T>()).ok()
    }

    pub fn type_info(&self) -> Type {
        self.as_state().type_info()
    }
}
