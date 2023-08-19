// #![deny(missing_docs)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use anathema_generator::DataCtx;
pub use anathema_render::Color;
use anathema_render::Style;
use anathema_values::{Container, List, PathId, ScopeValue, Truthy, ValueRef};

use crate::layout::{Align, Axis, Direction, Padding};
use crate::{ReadOnly, WidgetContainer};

// -----------------------------------------------------------------------------
//   - Cached -
// -----------------------------------------------------------------------------
fn cache_container<T: TryFrom<Value> + Clone>(
    container: Container<Value>,
    data: &DataCtx<'_, WidgetContainer>,
) -> Cached<T> {
    match container {
        Container::Value(val) => {
            let Ok(val) = T::try_from(val) else {
                return Cached::Empty;
            };
            Cached::Value(val)
        }
        Container::Empty => Cached::Empty,
        Container::List(list) => {
            let items = list
                .iter()
                .cloned()
                .filter_map(|val| data.by_ref(val))
                .map(|val| cache_container::<T>(val, data))
                .filter(Cached::not_empty)
                .collect::<Vec<_>>();
            Cached::List(items)
        }
    }
}

#[derive(Debug)]
pub enum Cached<T> {
    Empty,
    Value(T),
    Dyn {
        value_ref: ValueRef<Container<Value>>,
        value: Box<Cached<T>>,
    },
    List(Vec<Cached<T>>),
}

impl<T> Cached<T> {
    fn not_empty(&self) -> bool {
        match self {
            Self::Empty => false,
            _ => true,
        }
    }

    pub fn map<F, O>(&self, f: F) -> Option<O>
    where
        F: Fn(&T) -> O,
    {
        match self {
            Self::Empty => None,
            Self::List(_) => None,
            Self::Value(val) => Some(f(val)),
            Self::Dyn { value, .. } => value.map(f),
        }
    }
}

impl<T> Cached<T>
where
    T: TryFrom<Value> + Clone,
{
    pub fn new_or(key: &str, data: &DataCtx<'_, WidgetContainer>, alt: T) -> Self {
        match Cached::<T>::new(key, &data) {
            Cached::Empty => Cached::Value(alt),
            val @ _ => val,
        }
    }

    pub fn new(key: &str, data: &DataCtx<'_, WidgetContainer>) -> Self {
        let Some(scope_val) = data.get("display") else {
            return Self::Empty;
        };
        Cached::<T>::from_scope_val(scope_val, &data)
    }

    pub fn from_scope_val(source: ScopeValue<Value>, data: &DataCtx<'_, WidgetContainer>) -> Self {
        match source {
            ScopeValue::Static(val) => match T::try_from(val.deref().clone()) {
                Ok(val) => Self::Value(val),
                Err(_) => Self::Empty,
            },
            ScopeValue::List(ref list) => {
                let values = list
                    .iter()
                    .cloned()
                    .map(|sv| Cached::from_scope_val(sv, data))
                    .filter(Cached::not_empty)
                    .collect();

                Self::List(values)
            }
            ScopeValue::Dyn(value_ref) => {
                // Lookup the value and subscribe to it.
                // If the value points to another dyn value OR a list, evalute the value,
                // otherwise just Dyn { ref, value }

                match data.by_ref(value_ref) {
                    None => Cached::Empty,
                    Some(value) => Cached::Dyn {
                        value_ref,
                        value: Box::new(cache_container::<T>(value, data)),
                    },
                }
            }
        }
    }

    pub fn value_ref(&self) -> Option<&T> {
        match self {
            Self::Value(val) => Some(val),
            Self::Dyn { value, .. } => value.value_ref(),
            Self::Empty | Self::List(_) => None,
        }
    }

    fn update(&mut self, data: &ReadOnly) {}
}

impl<T> Cached<T>
where
    T: TryFrom<Value> + Copy,
{
    /// Single values only.
    /// This will panic if called on a list or a map
    pub fn or(&self, alt: T) -> T {
        match self {
            Self::Empty => alt,
            Self::Value(val) => *val,
            Self::Dyn { value, .. } => value.or(alt),
            Self::List(_) => panic!("called `or` on a list"),
        }
    }

    pub fn value(&self) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Value(val) => Some(*val),
            Self::Dyn { value, .. } => value.value(),
            Self::List(_) => panic!("called `or` on a list"),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Cached<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, ""),
            Self::Dyn { value, .. } => write!(f, "{value}"),
            Self::List(values) => values.iter().try_for_each(|value| write!(f, "{value}")),
            Self::Value(value) => write!(f, "{value}"),
        }
    }
}

/// Determine how a widget should be displayed and laid out
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Display {
    /// Show the widget, this is the default
    #[default]
    Show,
    /// Include the widget as part of the layout but don't render it
    Hide,
    /// Exclude the widget from the layout and paint step.
    Exclude,
}

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

impl Truthy for Number {
    fn is_true(&self) -> bool {
        match self {
            Self::Signed(n) => n.is_true(),
            Self::Unsigned(n) => n.is_true(),
            Self::Float(n) => n.is_true(),
        }
    }
}

/// A value.
#[derive(Debug, Clone)]
pub enum Value {
    /// Alignment.
    Alignment(Align),
    /// Axis.
    Axis(Axis),
    /// Boolean.
    Bool(bool),
    /// A colour.
    Color(Color),
    // /// A value lookup path.
    // DataBinding(PathId),
    /// Display is used to determine how to render and layout widgets.
    Display(Display),
    /// Direction
    Direction(Direction),
    /// A number.
    Number(Number),
    /// String: this is only available from the user data context.
    /// Strings generated from the parser is accessible only throught he `Text` lookup.
    String(String),
}

impl Truthy for Value {
    fn is_true(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::String(s) if s.is_empty() => false,
            // Self::List(list) => !list.is_empty(),
            // Self::Map(map) => panic!(),
            _ => true,
        }
    }
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

// impl<T: Into<Value>> From<Vec<T>> for Value {
//     fn from(v: Vec<T>) -> Self {
//         let values = v.into_iter().map(T::into).collect();
//         Value::List(values)
//     }
// }

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

macro_rules! impl_try_from {
    ($ret:ty, $variant:ident) => {
        impl TryFrom<Value> for $ret {
            type Error = ();

            fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::$variant(a) => Ok(a),
                    _ => Err(()),
                }
            }
        }

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

macro_rules! try_from_int {
    ($int:ty) => {
        impl TryFrom<Value> for $int {
            type Error = ();

            fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Unsigned(num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }

        impl<'a> TryFrom<&'a Value> for &'a $int {
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
        impl std::convert::TryFrom<Value> for $int {
            type Error = ();

            fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Signed(num)) => Ok(num),
                    _ => Err(()),
                }
            }
        }

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

impl TryFrom<Value> for usize {
    type Error = ();

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Unsigned(num)) => Ok(num as usize),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for isize {
    type Error = ();

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Signed(num)) => Ok(num as isize),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alignment(val) => write!(f, "{}", val),
            Self::Axis(val) => write!(f, "{:?}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::Color(val) => write!(f, "{:?}", val),
            Self::Display(val) => write!(f, "{:?}", val),
            Self::Direction(val) => write!(f, "{:?}", val),
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
