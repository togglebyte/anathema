// #![deny(missing_docs)]
use std::marker::PhantomData;

use crate::hashmap::HashMap;
use std::fmt;

use anathema_render::{Color};

// use crate::gen::store::Values;
use crate::{Display, Align, Axis, Direction, Fragment, path::PathId};
pub use crate::values2::{Map, List, ValueV2};

/// A value reference.
/// Used an index to lookup values
#[derive(Debug, PartialEq)]
pub struct ValueRef<T> {
    pub(crate) index: usize,
    pub(crate) gen: usize,
    _p: PhantomData<T>,
}

impl<T> ValueRef<T> {
    pub fn new(index: usize, gen: usize) -> Self {
        Self {
            index,
            gen,
            _p: PhantomData,
        }
    }

}

impl<T> Clone for ValueRef<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            gen: self.gen,
            _p: PhantomData
        }
    }
}

impl<T> Copy for ValueRef<T> {}

/// A number
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    /// Signed 64 bit number.
    Signed(i64),
    /// Unsigned 64 bit number.
    Unsigned(u64),
    /// 64 bit floating number.
    Float(f64),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Signed(num) => write!(f, "{}", num),
            Number::Unsigned(num) => write!(f, "{}", num),
            Number::Float(num) => write!(f, "{}", num),
        }
    }
}

/// A value.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Alignment.
    Alignment(Align),
    /// Axis.
    Axis(Axis),
    /// Boolean.
    Bool(bool),
    /// A colour.
    Color(Color),
    /// A value lookup path.
    DataBinding(PathId),
    /// Display is used to determine how to render and layout widgets.
    Display(Display),
    /// Direction
    Direction(Direction),
    /// An empty value.
    Empty,
    /// A list of values.
    List(Vec<Value>),
    /// A map of values.
    Map(HashMap<String, Value>),
    /// A number.
    Number(Number),
    /// String: this is only available from the user data context.
    /// Strings generated from the parser is accessible only throught he `Text` lookup.
    String(String),
    /// Fragments .
    Fragments(Vec<Fragment>),
}

// Implement `From` for an unsigned integer
macro_rules! from_int {
    ($int:ty) => {
        impl From<$int> for Value {
            fn from(v: $int) -> Self {
                Value::Number(Number::Unsigned(v as u64))
            }
        }
    };
    ($int:ty) => {
        impl From<&$int> for &Value {
            fn from(v: &$int) -> Self {
                Value::Number(Number::Unsigned(*v as u64))
            }
        }
    };
}

// Implement `From` for a signed integer
macro_rules! from_signed_int {
    ($int:ty) => {
        impl From<$int> for Value {
            fn from(v: $int) -> Self {
                Value::Number(Number::Signed(v as i64))
            }
        }
    };
    ($int:ty) => {
        impl From<&$int> for Value {
            fn from(v: &$int) -> Self {
                Value::Number(Number::Signed(*v as i64))
            }
        }
    };
}

from_int!(usize);
from_int!(u64);
from_int!(u32);
from_int!(u16);
from_int!(u8);

from_signed_int!(isize);
from_signed_int!(i64);
from_signed_int!(i32);
from_signed_int!(i16);
from_signed_int!(i8);

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(Number::Float(v))
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Number(Number::Float(v as f64))
    }
}

// impl<T: Into<Value>, U: Into<Value>> From<(T, U)> for Value {
//     fn from(tup: (T, U)) -> Self {
//         let (value_a, value_b) = (tup.0.into(), tup.1.into());
//         let hm = HashMap::from([("0".to_string(), value_a), ("1".to_string(), value_b)]);
//         Value::Map(hm)
//     }
// }

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        let values = v.into_iter().map(T::into).collect();
        Value::List(values)
    }
}

macro_rules! impl_from_val {
    ($t:ty, $variant:ident) => {
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Value::$variant(v)
            }
        }
    };
}

impl_from_val!(Align, Alignment);
impl_from_val!(Axis, Axis);
impl_from_val!(bool, Bool);
impl_from_val!(Color, Color);
impl_from_val!(Display, Display);
impl_from_val!(Number, Number);
impl_from_val!(String, String);
impl_from_val!(HashMap<String, Value>, Map);

macro_rules! impl_try_from {
    ($ret:ty, $variant:ident) => {
        impl<'a> TryFrom<&'a Value> for &'a $ret {
            type Error = ();

            fn try_from(value: &'a Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::$variant(ref a) => Ok(a),
                    _ => Err(()),
                }
            }
        }

        impl<'a> TryFrom<&'a mut Value> for &'a mut $ret {
            type Error = ();

            fn try_from(value: &'a mut Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::$variant(ref mut a) => Ok(a),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_try_from!(Align, Alignment);
impl_try_from!(Axis, Axis);
impl_try_from!(bool, Bool);
impl_try_from!(Color, Color);
impl_try_from!(Display, Display);
impl_try_from!(Number, Number);
impl_try_from!(String, String);
impl_try_from!(HashMap<String, Value>, Map);

macro_rules! try_from_int {
    ($int:ty) => {
        impl<'a> std::convert::TryFrom<&'a Value> for &'a $int {
            type Error = ();

            fn try_from(value: &'a Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Unsigned(ref num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a mut Value> for &'a mut $int {
            type Error = ();

            fn try_from(value: &'a mut Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Unsigned(ref mut num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_int!(u64);

macro_rules! try_from_signed_int {
    ($int:ty) => {
        impl<'a> std::convert::TryFrom<&'a Value> for &'a $int {
            type Error = ();

            fn try_from(value: &'a Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Signed(ref num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a mut Value> for &'a mut $int {
            type Error = ();

            fn try_from(value: &'a mut Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Signed(ref mut num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_signed_int!(i64);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, ""),
            Self::Alignment(val) => write!(f, "{}", val),
            Self::Axis(val) => write!(f, "{:?}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::Color(val) => write!(f, "{:?}", val),
            Self::DataBinding(val) => write!(f, "{:?}", val),
            Self::Display(val) => write!(f, "{:?}", val),
            Self::Direction(val) => write!(f, "{:?}", val),
            Self::Fragments(val) => write!(f, "Fragments {:?}", val),
            Self::List(val) => write!(f, "{:?}", val),
            Self::Map(val) => {
                write!(f, "{{ ")?;
                let s = val
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{s}")?;
                write!(f, " }}")?;
                Ok(())
            }
            Self::Number(val) => write!(f, "{}", val),
            Self::String(val) => write!(f, "{}", val),
        }
    }
}

impl Value {
    /// The value as an optional bool
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(val) => Some(*val),
            _ => None,
        }
    }

    /// The value as an optional string slice
    pub fn to_str(&self) -> Option<&str> {
        match self {
            Self::String(val) => Some(val),
            _ => None,
        }
    }

    /// The value as an optional path
    pub fn to_data_binding(&self) -> Option<&PathId> {
        match self {
            Self::DataBinding(val) => Some(val),
            _ => None,
        }
    }

    /// The value as an optional slice
    pub fn to_slice(&self) -> Option<&[Value]> {
        match self {
            Self::List(val) => Some(val.as_slice()),
            _ => None,
        }
    }

    /// The value as an optional map
    pub fn to_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Self::Map(val) => Some(val),
            _ => None,
        }
    }

    /// The value as an optional signed integer.
    /// This will cast any numerical value into an `i64`.
    /// This would be the equivalent of `number as i64`.
    ///
    /// If the value is a [`Value::Transition`] then this will use the boxed underlying value
    pub fn to_signed_int(&self) -> Option<i64> {
        match self {
            Self::Number(Number::Signed(val)) => Some(*val),
            Self::Number(Number::Unsigned(val)) => Some(*val as i64),
            Self::Number(Number::Float(val)) => Some(*val as i64),
            _ => None,
        }
    }

    /// The value as an optional unsigned integer.
    /// This will cast any numerical value into an `u64`.
    /// This would be the equivalent of `number as u64`.
    ///
    /// If the value is a [`Value::Transition`] then this will use the boxed underlying value
    pub fn to_int(&self) -> Option<u64> {
        match self {
            Self::Number(Number::Signed(val)) if *val >= 0 => Some(*val as u64),
            Self::Number(Number::Unsigned(val)) => Some(*val),
            Self::Number(Number::Float(val)) if *val >= 0.0 => Some(*val as u64),
            _ => None,
        }
    }

    /// The value as an optional unsigned integer.
    /// This will cast any numerical value into an `f64`.
    /// This would be the equivalent of `number as f64`.
    ///
    /// If the value is a [`Value::Transition`] then this will use the boxed underlying value
    pub fn to_float(&self) -> Option<f64> {
        match self {
            Self::Number(Number::Float(val)) => Some(*val),
            _ => None,
        }
    }

    /// The value as an optional alignment
    pub fn to_alignment(&self) -> Option<Align> {
        match self {
            Self::Alignment(val) => Some(*val),
            _ => None,
        }
    }

    /// The value as an optional color
    pub fn to_color(&self) -> Option<Color> {
        match self {
            Self::Color(col) => Some(*col),
            _ => None,
        }
    }

    /// The value as `Axis`
    pub fn to_axis(&self) -> Option<Axis> {
        match self {
            Self::Axis(axis) => Some(*axis),
            _ => None,
        }
    }

    /// The value as `Display`
    pub fn to_display(&self) -> Option<Display> {
        match self {
            Self::Display(disp) => Some(*disp),
            _ => None,
        }
    }

    /// The value as an optional string
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
