#![deny(missing_docs)]
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt;
use std::time::Duration;

use crate::display::Color;

use super::{Align, BorderStyle, Direction, Offset, Sides, TextAlignment, Wrap};
use crate::widgets::Display;

#[cfg(feature = "serde-json")]
pub mod json;

/// A `Fragment` can be either a [`Path`] or a `String`.
/// `Fragment`s are usually part of a list to represent a single string value.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Fragment {
    /// A string.
    String(String),
    /// A path to a value inside a context.
    Data(Path),
}

impl Fragment {
    /// Is the fragment a string?
    pub fn is_string(&self) -> bool {
        matches!(self, Fragment::String(_))
    }
}

/// A `Path` is used to look up a [`Value`] in a `crate::DataCtx`.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Path {
    /// Parent name
    pub name: String,
    /// Path to optional children
    pub child: Option<Box<Path>>,
}

impl Path {
    /// Create a new instance of a path.
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), child: None }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(child) = &self.child {
            write!(f, ".{}", child)?;
        }

        Ok(())
    }
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

/// Transition easing function.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Easing {
    /// Linear easing function. This is the default one.
    Linear,
    /// Ease in.
    EaseIn,
    /// Ease out.
    EaseOut,
    /// Ease in and out.
    EaseInOut,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}

impl Easing {
    pub(crate) fn apply(&self, time: f32) -> f32 {
        match self {
            Self::Linear => time,
            Self::EaseIn => 1.0 - (time * PI / 2.0).cos(),
            Self::EaseOut => ((time * PI) / 2.0).sin(),
            Self::EaseInOut => -((PI * time).cos() - 1.0) / 2.0,
        }
    }
}

/// A value.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Alignment.
    Alignment(Align),
    /// Direction.
    Direction(Direction),
    /// Boolean.
    Bool(bool),
    /// Border style, used with the [`crate::Border`] widget.
    BorderStyle(BorderStyle),
    /// A colour.
    Color(Color),
    /// A value lookup path.
    DataBinding(Path),
    /// Display is used to determine how to render and layout widgets.
    Display(Display),
    /// An empty value.
    Empty,
    /// A list of values.
    List(Vec<Value>),
    /// A map of values.
    Map(HashMap<String, Value>),
    /// A number.
    Number(Number),
    /// Border sides (determine which sides should be drawn).
    Sides(Sides),
    /// Offset (vertical / horizontal edges and offsets)
    Offset(Offset),
    /// String.
    String(String),
    /// Fragments .
    Fragments(Vec<Fragment>),
    /// Text alignment.
    TextAlignment(TextAlignment),
    /// Word wrapping.
    Wrap(Wrap),
    /// A transition.
    Transition(Box<Value>, Duration, Easing),
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
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

impl<T: Into<Value>, U: Into<Value>> From<(T, U)> for Value {
    fn from(tup: (T, U)) -> Self {
        let (value_a, value_b) = (tup.0.into(), tup.1.into());
        let hm = HashMap::from([("0".to_string(), value_a), ("1".to_string(), value_b)]);
        Value::Map(hm)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        let values = v.into_iter().map(T::into).collect::<Vec<_>>();
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
impl_from_val!(Direction, Direction);
impl_from_val!(bool, Bool);
impl_from_val!(BorderStyle, BorderStyle);
impl_from_val!(Color, Color);
impl_from_val!(Display, Display);
impl_from_val!(Number, Number);
impl_from_val!(Sides, Sides);
impl_from_val!(String, String);
impl_from_val!(TextAlignment, TextAlignment);
impl_from_val!(Wrap, Wrap);
impl_from_val!(Offset, Offset);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, ""),
            Self::Alignment(val) => write!(f, "{}", val),
            Self::Direction(val) => write!(f, "{:?}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::BorderStyle(val) => write!(f, "{:?}", val),
            Self::Color(val) => write!(f, "{:?}", val),
            Self::DataBinding(val) => write!(f, "{:?}", val),
            Self::Display(val) => write!(f, "{:?}", val),
            Self::Fragments(val) => write!(f, "Fragments {:?}", val),
            Self::List(val) => write!(f, "{:?}", val),
            Self::Map(val) => {
                write!(f, "{{ ")?;
                let s = val.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ");
                write!(f, "{s}")?;
                write!(f, " }}")?;
                Ok(())
            }
            Self::Number(val) => write!(f, "{}", val),
            Self::Sides(val) => write!(f, "{:?}", val),
            Self::String(val) => write!(f, "{}", val),
            Self::TextAlignment(val) => write!(f, "{:?}", val),
            Self::Wrap(val) => write!(f, "{:?}", val),
            Self::Offset(val) => write!(f, "{:?}", val),
            Self::Transition(val, duration, easing) => write!(f, "animate {val} over {duration:?} ms ({easing:?})"),
        }
    }
}

impl Value {
    /// Lookup a value inside a [`Value::Map`] using a [`Path`]
    pub fn lookup<'value>(path: &Path, data: &'value Value) -> Option<&'value Value> {
        match data {
            Value::Map(map) => match map.get(path.name.as_str())? {
                data @ Value::Map(_) => match &path.child {
                    Some(path) => Self::lookup(path, data),
                    None => Some(data),
                },
                val => Some(val),
            },
            _ => Some(data),
        }
    }

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
    pub fn to_data_binding(&self) -> Option<&Path> {
        match self {
            Self::DataBinding(val) => Some(val),
            _ => None,
        }
    }

    /// The value as an optional list
    pub fn to_list(&self) -> Option<&[Value]> {
        match self {
            Self::List(val) => Some(val),
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
            Self::Transition(value, _, _) => match value.as_ref() {
                Self::Number(Number::Signed(val)) => Some(*val),
                Self::Number(Number::Unsigned(val)) => Some(*val as i64),
                Self::Number(Number::Float(val)) => Some(*val as i64),
                _ => None,
            },
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
            Self::Transition(value, _, _) => match value.as_ref() {
                Self::Number(Number::Signed(val)) if *val >= 0 => Some(*val as u64),
                Self::Number(Number::Unsigned(val)) => Some(*val),
                Self::Number(Number::Float(val)) if *val >= 0.0 => Some(*val as u64),
                _ => None,
            },
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
            Self::Transition(value, _, _) => match value.as_ref() {
                Self::Number(Number::Float(val)) if *val >= 0.0 => Some(*val),
                _ => None,
            },
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

    /// The value as an optional text alignment
    pub fn to_text_align(&self) -> Option<TextAlignment> {
        match self {
            Self::TextAlignment(val) => Some(*val),
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

    /// The value as an optional string
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
