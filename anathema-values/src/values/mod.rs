use std::fmt::{self, Debug};
use std::sync::Arc;

pub use valueref::ValueRef;

pub use self::list::List;
// pub use self::map::Map;
use crate::store::StoreMut;
use crate::hashmap::{HashMap, IntMap};
use crate::Path;

mod list;
// mod map;
mod valueref;

/// Represent a value stored.
/// Both `Map` and `List` contains `ValueRef<T>` rather than `T`
#[derive(PartialEq)]
pub enum Container<T> {
    /// The empty value is used a placeholder. This makes it possible
    /// to associate a signal or such to a value that does not exist yet.
    Empty,
    Value(T),
    List(List<T>),
    // Map(Map<T>),
}

impl<T: Debug> Debug for Container<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Value::Empty"),
            Self::Value(val) => write!(f, "Value::Value({val:?})"),
            Self::List(list) => write!(f, "Value::List(<len: {}>)", list.len()),
        }
    }
}

// -----------------------------------------------------------------------------
//   - From value -
// -----------------------------------------------------------------------------
pub trait TryFromValue<T> {
    type Output;

    fn from_value(val: &Container<T>) -> Option<&Self::Output>;
}

// -----------------------------------------------------------------------------
//   - From value mut -
// -----------------------------------------------------------------------------
pub trait TryFromValueMut<T> {
    type Output;

    fn from_value(val: &mut Container<T>) -> Option<&mut Self::Output>;
}

// -----------------------------------------------------------------------------
//   - Into value -
// -----------------------------------------------------------------------------
pub trait IntoValue<T> {
    fn into_value(self, bucket: &mut StoreMut<'_, T>) -> Container<T>;
}

// Truthy
pub trait Truthy {
    fn is_true(&self) -> bool;
}

impl<T: Truthy> Truthy for Container<T> {
    fn is_true(&self) -> bool {
        match self {
            Container::Static(val) => val.is_true(),
            Container::List(l) => l.is_empty(),
            Container::Map(m) => m.is_empty(),
            _ => false,
        }
    }
}

impl Truthy for f64 {
    fn is_true(&self) -> bool {
        *self != 0.0
    }
}

macro_rules! int_impls {
    ($int:ty) => {
        impl Truthy for $int {
            fn is_true(&self) -> bool {
                *self != 0
            }
        }
    };
}

int_impls!(u8);
int_impls!(i8);
int_impls!(u16);
int_impls!(i16);
int_impls!(u32);
int_impls!(i32);
int_impls!(u64);
int_impls!(i64);
int_impls!(i128);
int_impls!(u128);
int_impls!(isize);
int_impls!(usize);

pub trait AsSlice {
    fn as_slice(&self) -> &[Self]
    where
        Self: Sized;
}
