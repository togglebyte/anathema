use std::fmt::{self, Debug};

pub use valueref::ValueRef;

pub use self::list::List;
pub use self::map::Map;
use crate::bucket::BucketMut;
use crate::hashmap::{HashMap, IntMap};
use crate::Path;

mod list;
mod map;
mod valueref;

/// Represent a value stored.
/// Both `Map` and `List` contains `ValueRef<T>` rather than `T`
#[derive(PartialEq)]
pub enum Container<T> {
    /// The empty value is used a placeholder. This makes it possible
    /// to associate a signal or such to a value that does not exist yet.
    Empty,
    Single(T),
    Map(Map<T>),
    List(List<T>),
}

impl<T: Debug> Debug for Container<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Value::Empty"),
            Self::Single(val) => write!(f, "Value::Single({val:?})"),
            Self::List(list) => write!(f, "Value::List(<len: {}>)", list.len()),
            Self::Map(map) => write!(f, "Value::Map(<len: {}>)", map.len()),
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

impl<T> TryFromValue<T> for T {
    type Output = T;

    fn from_value(val: &Container<T>) -> Option<&Self::Output> {
        match val {
            Container::Single(val) => Some(val),
            _ => None,
        }
    }
}

impl<T> TryFromValue<T> for List<T> {
    type Output = List<T>;

    fn from_value(val: &Container<T>) -> Option<&Self::Output> {
        match val {
            Container::List(list) => Some(list),
            _ => None,
        }
    }
}

impl<T> TryFromValue<T> for Map<T> {
    type Output = Map<T>;

    fn from_value(val: &Container<T>) -> Option<&Self::Output> {
        match val {
            Container::Map(map) => Some(map),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
//   - From value mut -
// -----------------------------------------------------------------------------
pub trait TryFromValueMut<T> {
    type Output;

    fn from_value(val: &mut Container<T>) -> Option<&mut Self::Output>;
}

impl<T> TryFromValueMut<T> for T {
    type Output = T;

    fn from_value(val: &mut Container<T>) -> Option<&mut Self::Output> {
        match val {
            Container::Single(val) => Some(val),
            _ => None,
        }
    }
}

impl<T> TryFromValueMut<T> for List<T> {
    type Output = List<T>;

    fn from_value(val: &mut Container<T>) -> Option<&mut Self::Output> {
        match val {
            Container::List(list) => Some(list),
            _ => None,
        }
    }
}

impl<T> TryFromValueMut<T> for Map<T> {
    type Output = Map<T>;

    fn from_value(val: &mut Container<T>) -> Option<&mut Self::Output> {
        match val {
            Container::Map(map) => Some(map),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Into value -
// -----------------------------------------------------------------------------
pub trait IntoValue<T> {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> Container<T>;
}

impl<T> IntoValue<T> for Container<T> {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> Container<T> {
        self
    }
}

// Single value
impl<T> IntoValue<T> for T {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> Container<T> {
        Container::Single(self)
    }
}

// List
impl<T> IntoValue<T> for Vec<T>
where
    T: IntoValue<T>,
{
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> Container<T> {
        let mut output = Vec::with_capacity(self.len());
        for val in self {
            let value_ref = bucket.push(val);
            output.push(value_ref);
        }
        Container::List(output.into())
    }
}

// Map
impl<K, V> IntoValue<V> for HashMap<K, V>
where
    V: IntoValue<V>,
    K: Into<Path>,
{
    fn into_value(self, bucket: &mut BucketMut<'_, V>) -> Container<V> {
        let mut output = IntMap::default();
        for (k, val) in self {
            let value_ref = bucket.push(val);
            let path_id = bucket.insert_path(k.into());
            output.insert(path_id.0, value_ref);
        }
        Container::Map(output.into())
    }
}

// Truthy
pub trait Truthy {
    fn is_true(&self) -> bool;
}

impl<T: Truthy> Truthy for Container<T> {
    fn is_true(&self) -> bool {
        match self {
            Container::Single(val) => val.is_true(),
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
