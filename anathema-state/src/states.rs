use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use anathema_store::slab::{Slab, SlabIndex};

use crate::{value::TypeId, CommonVal, Hex, Number, Path, PendingValue, Subscriber, Type, Value, ValueRef};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StateId(usize);

impl StateId {
    pub const ZERO: Self = Self(0);
}

impl SlabIndex for StateId {
    const MAX: usize = usize::MAX;

    fn as_usize(&self) -> usize {
        self.0
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index)
    }
}

pub trait AnyValue: 'static {
    fn type_id(&self) -> Type;

    fn to_any_ref(&self) -> &dyn Any;

    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn as_int(&self) -> Option<i64> {
        None
    }

    fn as_float(&self) -> Option<f64> {
        None
    }

    fn as_hex(&self) -> Option<Hex> {
        None
    }

    fn as_char(&self) -> Option<char> {
        None
    }

    fn as_str(&self) -> Option<&str> {
        None
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap> {
        None
    }

    fn as_any_list(&self) -> Option<&dyn AnyList> {
        None
    }
}

impl<T: AnyValue> AnyValue for Box<T> {
    fn type_id(&self) -> Type {
        self.as_ref().type_id()
    }

    fn to_any_ref(&self) -> &dyn Any {
        self.as_ref().to_any_ref()
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().to_any_mut()
    }

    fn as_int(&self) -> Option<i64> {
        self.as_ref().as_int()
    }

    fn as_float(&self) -> Option<f64> {
        self.as_ref().as_float()
    }

    fn as_char(&self) -> Option<char> {
        self.as_ref().as_char()
    }

    fn as_hex(&self) -> Option<Hex> {
        self.as_ref().as_hex()
    }

    fn as_str(&self) -> Option<&str> {
        self.as_ref().as_str()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_ref().as_bool()
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap> {
        self.as_ref().as_any_map()
    }

    fn as_any_list(&self) -> Option<&dyn AnyList> {
        self.as_ref().as_any_list()
    }
}

impl AnyValue for Box<dyn AnyValue> {
    fn type_id(&self) -> Type {
        self.as_ref().type_id()
    }

    fn to_any_ref(&self) -> &dyn Any {
        self.as_ref().to_any_ref()
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().to_any_mut()
    }

    fn as_int(&self) -> Option<i64> {
        self.as_ref().as_int()
    }

    fn as_float(&self) -> Option<f64> {
        self.as_ref().as_float()
    }

    fn as_char(&self) -> Option<char> {
        self.as_ref().as_char()
    }

    fn as_hex(&self) -> Option<Hex> {
        self.as_ref().as_hex()
    }

    fn as_str(&self) -> Option<&str> {
        self.as_ref().as_str()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_ref().as_bool()
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap> {
        self.as_ref().as_any_map()
    }

    fn as_any_list(&self) -> Option<&dyn AnyList> {
        self.as_ref().as_any_list()
    }
}

pub trait AnyMap {
    fn lookup(&self, key: std::borrow::Cow<'_, str>) -> Option<PendingValue>;
}

pub trait AnyList {
    fn lookup(&self, index: usize) -> Option<PendingValue>;
}

// -----------------------------------------------------------------------------
//   - Old state -
// -----------------------------------------------------------------------------

pub trait AnyState: 'static {
    fn to_any_ref(&self) -> &dyn Any;

    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn to_common(&self) -> Option<CommonVal>;

    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef>;

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue>;

    fn to_number(&self) -> Option<Number>;

    fn to_bool(&self) -> bool;

    fn count(&self) -> usize;
}

impl AnyState for Box<dyn AnyState> {
    fn to_any_ref(&self) -> &dyn Any {
        self.as_ref().to_any_ref()
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().to_any_mut()
    }

    fn to_common(&self) -> Option<CommonVal> {
        self.as_ref().to_common()
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

    fn count(&self) -> usize {
        self.as_ref().count()
    }
}

impl<T: State> AnyState for T {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_common(&self) -> Option<CommonVal> {
        <Self as State>::to_common(self)
    }

    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        <Self as State>::state_get(self, path, sub)
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        <Self as State>::state_lookup(self, path)
    }

    fn to_number(&self) -> Option<Number> {
        <Self as State>::to_number(self)
    }

    fn to_bool(&self) -> bool {
        <Self as State>::to_bool(self)
    }

    fn count(&self) -> usize {
        <Self as State>::count(self)
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
    fn count(&self) -> usize {
        0
    }

    fn to_number(&self) -> Option<Number> {
        None
    }

    fn to_bool(&self) -> bool {
        false
    }

    fn to_common(&self) -> Option<CommonVal>;
}

impl State for Box<dyn State> {
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

    fn to_common(&self) -> Option<CommonVal> {
        self.as_ref().to_common()
    }

    fn count(&self) -> usize {
        self.as_ref().count()
    }
}

// impl<T: 'static + State> State for Value<T> {
//     fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
//         self.to_ref().state_get(path, sub)
//     }

//     fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
//         self.to_ref().state_lookup(path)
//     }

//     fn to_number(&self) -> Option<Number> {
//         self.to_ref().to_number()
//     }

//     fn to_bool(&self) -> bool {
//         self.to_ref().to_bool()
//     }

//     fn to_common(&self) -> Option<CommonVal> {
//         None
//     }

//     fn count(&self) -> usize {
//         self.to_ref().count()
//     }
// }

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
        impl AnyValue for $t {
            fn type_id(&self) -> Type {
                Type::Int
            }

            fn to_any_ref(&self) -> &dyn Any {
                self
            }

            fn to_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn as_int(&self) -> Option<i64> {
                Some(*self as i64)
            }
        }

        impl State for $t {
            fn to_number(&self) -> Option<Number> {
                Number::try_from(*self).ok()
            }

            fn to_bool(&self) -> bool {
                <Self as State>::to_number(self)
                    .map(|n| n.as_int() != 0)
                    .unwrap_or(false)
            }

            fn to_common(&self) -> Option<CommonVal> {
                Some(CommonVal::Int(*self as i64))
            }
        }
    };
}

macro_rules! impl_float_state {
    ($t:ty) => {
        impl AnyValue for $t {
            fn type_id(&self) -> Type {
                Type::Float
            }

            fn to_any_ref(&self) -> &dyn Any {
                self
            }

            fn to_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn as_float(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }

        impl State for $t {
            fn to_number(&self) -> Option<Number> {
                Number::try_from(*self).ok()
            }

            fn to_bool(&self) -> bool {
                <Self as State>::to_number(self)
                    .map(|n| n.as_int() != 0)
                    .unwrap_or(false)
            }

            fn to_common(&self) -> Option<CommonVal> {
                Some(CommonVal::Float(*self as f64))
            }
        }
    };
}

impl AnyValue for bool {
    fn type_id(&self) -> Type {
        Type::Bool
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }
}

impl AnyValue for String {
    fn type_id(&self) -> Type {
        Type::String
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl AnyValue for &'static str {
    fn type_id(&self) -> Type {
        Type::String
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_str(&self) -> Option<&str> {
        Some(*self)
    }
}

impl AnyValue for char {
    fn type_id(&self) -> Type {
        Type::Char
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_char(&self) -> Option<char> {
        Some(*self)
    }
}

impl AnyValue for Hex {
    fn type_id(&self) -> Type {
        Type::Char
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_hex(&self) -> Option<Hex> {
        Some(*self)
    }
}

impl AnyValue for () {
    fn type_id(&self) -> Type {
        Type::Composite
    }

    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl State for bool {
    fn to_bool(&self) -> bool {
        *self
    }

    fn to_common(&self) -> Option<CommonVal> {
        Some(CommonVal::Bool(*self))
    }
}

impl State for Hex {
    fn to_common(&self) -> Option<CommonVal> {
        Some(CommonVal::Hex(*self))
    }
}

impl State for char {
    fn to_common(&self) -> Option<CommonVal> {
        Some(CommonVal::Char(*self))
    }
}

impl State for () {
    fn to_common(&self) -> Option<CommonVal> {
        None
    }
}

impl<T: State> State for Option<T> {
    fn to_common(&self) -> Option<CommonVal> {
        self.as_ref()?.to_common()
    }

    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        self.as_ref()?.state_get(path, sub)
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        self.as_ref()?.state_lookup(path)
    }

    fn count(&self) -> usize {
        self.as_ref().map(|s| s.count()).unwrap_or(0)
    }

    fn to_number(&self) -> Option<Number> {
        self.as_ref()?.to_number()
    }

    fn to_bool(&self) -> bool {
        self.as_ref().map(|s| s.to_bool()).unwrap_or(false)
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
impl_num_state!(usize);
impl_float_state!(f32);
impl_float_state!(f64);

pub struct States {
    inner: Slab<StateId, Value<Box<dyn AnyValue>>>,
}

impl States {
    pub fn new() -> Self {
        Self { inner: Slab::empty() }
    }

    // pub fn insert(&mut self, state: Value<Box<dyn AnyValue>>) -> StateId {
    pub fn insert(&mut self, state: impl AnyValue) -> StateId {
        let state: Value<Box<dyn AnyValue>> = Value::new(Box::new(state));
        self.inner.insert(state)
    }

    pub fn get(&self, state_id: impl Into<StateId>) -> Option<&Value<Box<dyn AnyValue>>> {
        self.inner.get(state_id.into()).map(|b| b)
    }

    pub fn get_mut(&mut self, state_id: impl Into<StateId>) -> Option<&mut Value<Box<dyn AnyValue>>> {
        self.inner.get_mut(state_id.into())
    }

    // pub fn with_mut<F, U>(&mut self, index: impl Into<StateId>, f: F) -> U
    // where
    //     F: FnOnce(&mut dyn AnyState, &mut Self) -> U,
    // {
    //     let mut ticket = self.inner.checkout(index.into());
    //     let ret = f(&mut *ticket, self);
    //     self.inner.restore(ticket);
    //     ret
    // }

    /// Remove and return a given state.
    ///
    /// # Panics
    ///
    /// Will panic if the state does not exist.
    pub fn remove(&mut self, state_id: StateId) -> Value<Box<dyn AnyValue>> {
        self.inner.remove(state_id)
    }
}
