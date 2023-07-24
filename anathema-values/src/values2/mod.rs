use std::fmt::{Debug, self};
use crate::bucket::BucketMut;
use crate::hashmap::IntMap;

pub use self::list::List;
pub use self::map::Map;

mod list;
mod map;

/// Represent a value stored.
/// Both `Map` and `List` contains `ValueRef<T>` rather than `T`
#[derive(PartialEq)]
pub enum ValueV2<T> {
    Single(T),
    Map(Map<T>),
    List(List<T>),
}

impl<T: Debug> Debug for ValueV2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(val) => write!(f, "Value::Single({val:?})"),
            Self::List(list) => write!(f, "Value::List(<list: {}>)", list.len()),
            Self::Map(map) => write!(f, "Value::Map(<map: {}>)", map.len()),
        }
    }
}

// -----------------------------------------------------------------------------
//   - From value -
// -----------------------------------------------------------------------------
pub trait TryFromValue<T> {
    type Output;

    fn from_value(val: &ValueV2<T>) -> Option<&Self::Output>;
}

impl<T> TryFromValue<T> for T {
    type Output = T;

    fn from_value(val: &ValueV2<T>) -> Option<&Self::Output> {
        match val {
            ValueV2::Single(val) => Some(val),
            _ => None
        }
    }
}

impl<T> TryFromValue<T> for List<T> {
    type Output = List<T>;

    fn from_value(val: &ValueV2<T>) -> Option<&Self::Output> {
        match val {
            ValueV2::List(list) => Some(list),
            _ => None
        }
    }
}

impl<T> TryFromValue<T> for Map<T> {
    type Output = Map<T>;

    fn from_value(val: &ValueV2<T>) -> Option<&Self::Output> {
        match val {
            ValueV2::Map(map) => Some(map),
            _ => None
        }
    }
}

// -----------------------------------------------------------------------------
//   - Into value -
// -----------------------------------------------------------------------------
pub trait IntoValue<T> {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> ValueV2<T>;
}

// Single value
impl<T> IntoValue<T> for T {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> ValueV2<T> {
        ValueV2::Single(self)
    }
}

// List
impl<T> IntoValue<T> for Vec<T> where T: IntoValue<T> {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> ValueV2<T> {
        let mut output = Vec::with_capacity(self.len());
        for val in self {
            let value_ref = bucket.push(val);
            output.push(value_ref);
        }
        ValueV2::List(output.into())
    }
}

// Map
impl<T> IntoValue<T> for IntMap<String, T> where T: IntoValue<T> {
    fn into_value(self, bucket: &mut BucketMut<'_, T>) -> ValueV2<T> {
        let mut output = IntMap::default();
        for (k, val) in self {
            let value_ref = bucket.push(val);
            let path_id = bucket.insert_path(k);
            output.insert(path_id.0, value_ref);
        }
        ValueV2::Map(output.into())
    }
}
