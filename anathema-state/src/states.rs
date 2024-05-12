use std::any::Any;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use anathema_store::slab::Slab;

use crate::{CommonVal, Hex, Number, Path, PendingValue, Subscriber, Value, ValueRef};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StateId(usize);

impl StateId {
    pub const ZERO: Self = Self(0);
}

impl From<usize> for StateId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<StateId> for usize {
    fn from(value: StateId) -> Self {
        value.0
    }
}

pub trait State: 'static {
    /// Try to get the value from the state.
    /// If the value exists: subscribe to the value with the key and return
    /// the value ref
    fn state_get(&self, _path: Path<'_>, _sub: Subscriber) -> Option<ValueRef> {
        None
    }

    /// Lookup a value by a path.
    /// Unlike `state_get` this does not require a key to be associated
    /// with the value.
    fn state_lookup(&self, _path: Path<'_>) -> Option<PendingValue> {
        None
    }

    /// Get the length of any underlying collection.
    /// If the state is not a collection it should return zero
    fn len(&self) -> usize {
        0
    }

    fn to_any_ref(&self) -> &dyn Any;

    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn to_number(&self) -> Option<Number> {
        None
    }

    fn to_bool(&self) -> bool {
        false
    }

    fn to_common(&self) -> Option<CommonVal<'_>>;
}

impl State for Box<dyn State> {
    fn to_any_ref(&self) -> &dyn Any {
        self.as_ref().to_any_ref()
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().to_any_mut()
    }

    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        self.as_ref().state_get(path, sub)
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        self.as_ref().state_lookup(path)
    }

    fn to_number(&self) -> Option<Number> {
        self.as_ref().to_number()
    }

    fn to_bool(&self) -> bool {
        self.as_ref().to_bool()
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        self.as_ref().to_common()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T: 'static + State> State for Value<T> {
    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        self.to_ref().state_get(path, sub)
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        self.to_ref().state_lookup(path)
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_number(&self) -> Option<Number> {
        self.to_ref().to_number()
    }

    fn to_bool(&self) -> bool {
        self.to_ref().to_bool()
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }

    fn len(&self) -> usize {
        self.to_ref().len()
    }
}

impl Debug for dyn State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<dyn State>")
    }
}

// -----------------------------------------------------------------------------
//   - State implementation... -
//   State implementation for primitives and non-state types
// -----------------------------------------------------------------------------
macro_rules! impl_num_state {
    ($t:ty) => {
        impl State for $t {
            fn to_any_ref(&self) -> &dyn Any {
                self
            }

            fn to_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn to_number(&self) -> Option<Number> {
                Number::try_from(*self).ok()
            }

            fn to_bool(&self) -> bool {
                self.to_number().map(|n| n.as_int() != 0).unwrap_or(false)
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                Some(CommonVal::Int(*self as i64))
            }
        }
    };
}

macro_rules! impl_str_state {
    ($t:ty) => {
        impl State for $t {
            fn to_any_ref(&self) -> &dyn Any {
                self
            }

            fn to_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn to_bool(&self) -> bool {
                !self.is_empty()
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                Some(CommonVal::Str(&*self))
            }
        }
    };
}

impl State for bool {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_bool(&self) -> bool {
        *self
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Bool(*self))
    }
}

impl State for Hex {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Hex(*self))
    }
}

impl State for () {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }
}

impl_num_state!(u8);
impl_num_state!(i8);
impl_num_state!(u16);
impl_num_state!(i16);
impl_num_state!(u32);
impl_num_state!(i32);
impl_num_state!(u64);
impl_num_state!(i64);
impl_num_state!(f32);
impl_num_state!(f64);
impl_num_state!(usize);
impl_str_state!(String);
impl_str_state!(&'static str);
impl_str_state!(Box<str>);
impl_str_state!(Rc<str>);

pub struct States {
    inner: Slab<StateId, Box<dyn State>>,
}

impl States {
    pub fn new() -> Self {
        Self { inner: Slab::empty() }
    }

    pub fn insert(&mut self, state: Box<dyn State>) -> StateId {
        self.inner.insert(state)
    }

    pub fn get(&self, state_id: impl Into<StateId>) -> Option<&dyn State> {
        self.inner.get(state_id.into()).map(|b| &**b)
    }

    pub fn get_mut(&mut self, state_id: impl Into<StateId>) -> Option<&mut dyn State> {
        self.inner.get_mut(state_id.into()).map(|b| {
            let state: &mut dyn State = &mut *b;
            state
        })
    }
}

pub struct Stateless<T>(Value<NoState<T>>);

impl<T: 'static> Stateless<T> {
    pub fn new(value: T) -> Self {
        Self::from(value)
    }
}

impl<T> Deref for Stateless<T> {
    type Target = Value<NoState<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Stateless<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: 'static> State for Stateless<T> {
    fn to_any_ref(&self) -> &dyn Any {
        &self.0
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }
}

impl<T: 'static> From<T> for Stateless<T> {
    fn from(value: T) -> Self {
        Self(Value::new(NoState(value)))
    }
}

/// A stateless value wrapper
pub struct NoState<T>(pub T);

impl<T: 'static> State for NoState<T> {
    fn to_any_ref(&self) -> &dyn Any {
        &self.0
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }
}

impl<T> Deref for NoState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NoState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
