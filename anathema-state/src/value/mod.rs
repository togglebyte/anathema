use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use anathema_store::slab::Element;
use anathema_store::store::{OwnedKey, SharedKey};

pub use self::list::List;
pub use self::map::Map;
use super::State;
use crate::states::AnyState;
use crate::store::subscriber::{subscribe, unsubscribe};
use crate::store::values::{
    copy_val, drop_value, get_unique, make_shared, new_value, return_owned, return_shared, try_make_shared, with_owned,
};
use crate::store::{changed, ValueKey};
use crate::{Change, Subscriber};

mod list;
mod map;

/// A value that reacts to change.
///
/// The value is stored in a global store and accessed via the `Value`.
/// ```
/// # use anathema_state::*;
/// let mut value = Value::<usize>::new(1);
/// *value.to_mut() += 1;
/// ```
#[derive(Debug)]
pub struct Value<T> {
    key: ValueKey,
    // Ensure that `Value` is not Send or Sync.
    // Given that values live in TLS, sending a value across thread boundaries
    // would result in loading an incorrect value
    _p: PhantomData<*const T>,
}

impl<T: Default + AnyState + 'static> Default for Value<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: AnyState + 'static> From<T> for Value<T> {
    fn from(value: T) -> Self {
        Value::new(value)
    }
}

impl<T: AnyState + 'static> Value<T> {
    /// Create a new instance of a `Value`.
    pub fn new(value: T) -> Self {
        let key = new_value(Box::new(value));

        Self { key, _p: PhantomData }
    }

    /// A `Unique` reference to the value.
    /// There can only be one of these at any given point for a given `Value`.
    ///
    /// Attempting to take a reference to the value using a `ValueRef` will
    /// result in a runtime error.
    pub fn to_mut(&mut self) -> Unique<'_, T> {
        let value = get_unique(self.key.owned());
        Unique {
            value: Some(value),
            key: self.key,
            _p: PhantomData,
        }
    }

    /// A `Shared` reference to the value.
    /// There can be several shared references to a given value as long as there
    /// is no unique access to the value.
    #[must_use]
    pub fn to_ref(&self) -> Shared<'_, T> {
        let (key, value) = make_shared(self.key.owned()).expect("the value exists as it's coming directly from `Self`");

        Shared {
            state: SharedState::new(key, value),
            _p: PhantomData,
        }
    }

    /// Produce a detached `ValueRef`.
    /// Since this is not subject to the same lifetime as the `Value` it originates from it is
    /// possible to try to access the underlying value while a `Unique` reference exists.
    /// This will result in a runtime error.
    #[must_use]
    pub fn value_ref(&self, subscriber: Subscriber) -> ValueRef {
        subscribe(self.key.sub(), subscriber);
        ValueRef {
            value_key: self.key,
            subscriber,
        }
    }

    /// A Pending value that can later become a value reference when
    /// combined with a subscriber.
    pub fn to_pending(&self) -> PendingValue {
        PendingValue(self.key)
    }

    pub fn shared_state(&self) -> Option<SharedState<'_>> {
        let (key, value) = try_make_shared(self.key.owned())?;
        let shared = SharedState::new(key, value);
        Some(shared)
    }

    /// Get a copy of the value key.
    /// Useful for debugging.
    pub fn key(&self) -> ValueKey {
        self.key
    }

    /// Convenience function for reassigning a value.
    pub fn set(&mut self, new_value: T) {
        *self.to_mut() = new_value;
    }
}

/// Copy the inner value from the owned value.
impl<T: State + 'static + Copy> Value<T> {
    pub fn copy_value(&self) -> T {
        copy_val(self.key.owned())
    }
}

impl<T> Drop for Value<T> {
    fn drop(&mut self) {
        changed(self.key.sub(), Change::Dropped);
        drop_value(self.key);
    }
}

/// Unique access to the underlying value.
/// This is the primary means to mutate the value.
pub struct Unique<'a, T: 'static> {
    value: Option<Box<dyn AnyState>>,
    key: ValueKey,
    _p: PhantomData<&'a mut T>,
}

impl<'a, T> Deref for Unique<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
            .as_ref()
            .expect("value is only ever set to None on drop")
            .to_any_ref()
            .downcast_ref()
            .expect("the type should never change")
    }
}

impl<'a, T> DerefMut for Unique<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        changed(self.key.sub(), Change::Changed);
        self.value
            .as_mut()
            .expect("value is only ever set to None on drop")
            .to_any_mut()
            .downcast_mut()
            .expect("the type should never change")
    }
}

impl<'a, T: 'static> Drop for Unique<'a, T> {
    fn drop(&mut self) {
        // TODO this can be an unwrap_unchecked because the `value` is always Some(_) in `Unique`
        //      except here where it's dropped
        let value = self.value.take().unwrap();

        // this is the only place where self.value = None
        return_owned(self.key.owned(), value);
    }
}

// -----------------------------------------------------------------------------
//   - Shared -
// -----------------------------------------------------------------------------
#[derive(Default)]
enum ElementState {
    Alive(Element<Box<dyn AnyState>>),
    #[default]
    Dropped,
}

impl ElementState {
    #[allow(clippy::borrowed_box)]
    fn as_state(&self) -> &Box<dyn AnyState> {
        match self {
            Self::Dropped => unreachable!(),
            Self::Alive(ref value) => value,
        }
    }

    fn as_ref<T: 'static>(&self) -> &T {
        match self {
            Self::Dropped => unreachable!(),
            Self::Alive(ref value) => value.to_any_ref().downcast_ref().expect("invalid type"),
        }
    }

    fn try_as_ref<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Dropped => unreachable!(),
            Self::Alive(ref value) => value.to_any_ref().downcast_ref(),
        }
    }

    fn drop_value(&mut self) {
        let _ = std::mem::take(self);
    }
}

pub struct Shared<'a, T: 'static> {
    state: SharedState<'a>,
    _p: PhantomData<T>,
}

impl<'a, T> Shared<'a, T> {
    fn new(key: SharedKey, value: Element<Box<dyn AnyState>>) -> Self {
        Self {
            state: SharedState::new(key, value),
            _p: PhantomData,
        }
    }

    pub fn try_as_ref(&self) -> Option<&T> {
        self.state.inner.try_as_ref()
    }
}

impl<'a, T> Deref for Shared<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state.inner.as_ref()
    }
}

impl<T> AsRef<T> for Shared<'_, T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T: Debug> Debug for Shared<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self
            .state
            .inner
            .as_state()
            .to_any_ref()
            .downcast_ref::<T>()
            // It's fine to expect here since the type information
            // is retained from the source that produces the `Shared` instance.
            .expect("type information is retained");
        f.debug_struct("Shared").field("state", &state).finish()
    }
}

/// Shared state
pub struct SharedState<'a> {
    inner: ElementState,
    key: SharedKey,
    _p: PhantomData<&'a ()>,
}

impl<'a> SharedState<'a> {
    fn new(key: SharedKey, state: Element<Box<dyn AnyState>>) -> Self {
        Self {
            key,
            inner: ElementState::Alive(state),
            _p: PhantomData,
        }
    }
}

impl PartialEq for SharedState<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<'a> Deref for SharedState<'a> {
    type Target = Box<dyn AnyState>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_state()
    }
}

impl<'a> Drop for SharedState<'a> {
    fn drop(&mut self) {
        self.inner.drop_value();
        return_shared(self.key);
    }
}

/// This is a detached value without an associated value type.
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
/// # let key_1 = Subscriber::ZERO;
/// # let key_2 = Subscriber::MAX;
/// let value = Value::new(123u32);
/// let v1 = value.value_ref(key_1);
/// let v2 = value.value_ref(key_2);
///
/// assert_eq!(*v1.value::<u32>().unwrap(), 123);
/// ```
#[derive(Debug, PartialEq)]
pub struct ValueRef {
    value_key: ValueKey,
    subscriber: Subscriber,
}

impl ValueRef {
    /// Load the value. This will return `None` if the owner has dropped the value
    pub fn value<T: 'static>(&self) -> Option<Shared<'_, T>> {
        let (key, value) = try_make_shared(self.value_key.owned())?;
        let shared = Shared::new(key, value);
        Some(shared)
    }

    /// Try to get access to the underlying value as a `dyn AnyState`.
    /// This will return `None` if the `Value<T>` behind this `ValueRef` has
    /// been dropped.
    pub fn as_state(&self) -> Option<SharedState<'_>> {
        let (key, value) = try_make_shared(self.value_key.owned())?;
        let shared = SharedState::new(key, value);
        Some(shared)
    }

    pub fn to_pending(&self) -> PendingValue {
        PendingValue(self.value_key)
    }

    /// Get a copy of the owned key.
    /// Used for debugging.
    pub fn owned_key(&self) -> OwnedKey {
        self.value_key.owned()
    }

    pub fn copy_with_sub(&self, subscriber: Subscriber) -> ValueRef {
        subscribe(self.value_key.sub(), subscriber);
        ValueRef {
            value_key: self.value_key,
            subscriber,
        }
    }
}

impl Drop for ValueRef {
    fn drop(&mut self) {
        unsubscribe(self.value_key.sub(), self.subscriber);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PendingValue(ValueKey);

impl PendingValue {
    pub fn to_value(&self, subscriber: Subscriber) -> ValueRef {
        subscribe(self.0.sub(), subscriber);
        ValueRef {
            value_key: self.0,
            subscriber,
        }
    }

    pub fn as_state<F, T>(&self, f: F) -> T
    where
        F: Fn(&dyn AnyState) -> T,
    {
        with_owned(self.0.owned(), f)
    }

    pub fn owned_key(&self) -> OwnedKey {
        self.0.owned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_value() {
        let mut value = Value::new("hello world");
        let unique = value.to_mut();
        assert_eq!("hello world", *unique);
    }

    #[test]
    fn mutable_access() {
        let mut value = Value::new(String::new());
        {
            let mut unique = value.to_mut();
            unique.push_str("updated");
        }

        let unique = value.to_mut();
        assert_eq!("updated", *unique);
    }

    #[test]
    fn shared_access() {
        let expected = "hello world";
        let value = Value::new(expected);
        let s1 = value.to_ref();
        let s2 = value.to_ref();

        assert_eq!(*s1, expected);
        assert_eq!(*s2, expected);
    }

    #[test]
    #[should_panic(expected = "value is currently shared: OwnedKey(0)")]
    fn mutable_shared_panic() {
        // This hould panic because of mutable access
        // is held while also having a value reference.
        let mut value = Value::new(String::new());
        let s1 = value.value_ref(Subscriber::ZERO);
        let _r1 = s1.value::<String>();
        let _m1 = value.to_mut();
    }

    #[test]
    fn value_ref_to_shared_state() {
        let value = Value::new(1);
        let r1 = value.value_ref(Subscriber::ZERO);
        let r2 = value.value_ref(Subscriber::ZERO);

        let s1 = r1.as_state().unwrap();
        let s2 = r2.as_state().unwrap();

        let val = s1.to_number().unwrap() + s2.to_number().unwrap();

        assert_eq!(val.as_int(), 2);
    }
}
