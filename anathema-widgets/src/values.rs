// #![deny(missing_docs)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{self, Write};
use std::time::Duration;

use anathema_render::{Size, Style};

use crate::gen::store::Store;
use crate::gen::ValueRef;
use crate::{
    fields, Align, Attributes, Axis, Color, DataCtx, Direction, Display, Padding, Path,
    TextPath, Wrap,
};
// use crate::{
//     Align, Axis, BorderStyle, Color, Direction, Display, Offset, Path, Sides, TextAlignment, Wrap,
// };

#[cfg(feature = "serde-json")]
pub mod json;

/// Text alignment aligns the text inside its parent.
///
/// Given a border with a width of nine and text alignment set to [`TextAlignment::Right`]:
/// ```text
/// ┌───────┐
/// │I would│
/// │ like a│
/// │ lovely│
/// │ cup of│
/// │    tea│
/// │ please│
/// └───────┘
/// ```
///
/// The text will only align it self within the parent widget.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextAlignment {
    /// Align the to the left inside the parent
    Left,
    /// Align the text in the centre of the parent
    Centre,
    /// Align the to the right inside the parent
    Right,
}

// -----------------------------------------------------------------------------
//   - Border types -
// -----------------------------------------------------------------------------
pub const DEFAULT_SLIM_EDGES: [char; 8] = ['┌', '─', '┐', '│', '┘', '─', '└', '│'];
pub const DEFAULT_THICK_EDGES: [char; 8] = ['╔', '═', '╗', '║', '╝', '═', '╚', '║'];

// -----------------------------------------------------------------------------
//     - Indices -
//     Index into `DEFAULT_SLIM_EDGES` or `DEFAULT_THICK_EDGES`
// -----------------------------------------------------------------------------
pub const BORDER_EDGE_TOP_LEFT: usize = 0;
pub const BORDER_EDGE_TOP: usize = 1;
pub const BORDER_EDGE_TOP_RIGHT: usize = 2;
pub const BORDER_EDGE_RIGHT: usize = 3;
pub const BORDER_EDGE_BOTTOM_RIGHT: usize = 4;
pub const BORDER_EDGE_BOTTOM: usize = 5;
pub const BORDER_EDGE_BOTTOM_LEFT: usize = 6;
pub const BORDER_EDGE_LEFT: usize = 7;

// -----------------------------------------------------------------------------
//     - Sides -
// -----------------------------------------------------------------------------
bitflags::bitflags! {
    /// Border sides
    /// ```
    /// use anathema_widgets::Sides;
    /// let sides = Sides::TOP | Sides::LEFT;
    /// ```
    pub struct Sides: u8 {
        /// Empty
        const EMPTY = 0x0;
        /// Top border
        const TOP = 0b0001;
        /// Right border
        const RIGHT = 0b0010;
        /// Bottom border
        const BOTTOM = 0b0100;
        /// Left border
        const LEFT = 0b1000;
        /// All sides
        const ALL = Self::TOP.bits | Self::RIGHT.bits | Self::BOTTOM.bits | Self::LEFT.bits;
    }
}

/// The style of the border.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorderStyle {
    /// ```text
    /// ┌─────┐
    /// │hello│
    /// └─────┘
    /// ```
    Thin,
    /// ```text
    /// ╔═════╗
    /// ║hello║
    /// ╚═════╝
    /// ```
    Thick,
    /// ```text
    /// 0111112
    /// 7hello3
    /// 6555554
    /// ```
    Custom(String),
}

impl BorderStyle {
    pub(crate) fn edges(&self) -> [char; 8] {
        match self {
            BorderStyle::Thin => DEFAULT_SLIM_EDGES,
            BorderStyle::Thick => DEFAULT_THICK_EDGES,
            BorderStyle::Custom(edge_string) => {
                let mut edges = [' '; 8];
                for (i, c) in edge_string.chars().take(8).enumerate() {
                    edges[i] = c;
                }
                edges
            }
        }
    }
}

/// A `Fragment` can be either a [`Path`] or a `String`.
/// `Fragment`s are usually part of a list to represent a single string value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// Axis.
    Axis(Axis),
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
    /// Border sides (determine which sides should be drawn).
    Sides(Sides),
    /// String: this is only available from the user data context.
    /// Strings generated from the parser is accessible only throught he `Text` lookup.
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

impl<T: Into<Value>, U: Into<Value>> From<(T, U)> for Value {
    fn from(tup: (T, U)) -> Self {
        let (value_a, value_b) = (tup.0.into(), tup.1.into());
        let hm = HashMap::from([("0".to_string(), value_a), ("1".to_string(), value_b)]);
        Value::Map(hm)
    }
}

impl<K: Into<String>, V: Into<Value>> From<HashMap<K, V>> for Value {
    fn from(v: HashMap<K, V>) -> Self {
        let values = v.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Value::Map(values)
    }
}

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
impl_from_val!(BorderStyle, BorderStyle);
impl_from_val!(Color, Color);
impl_from_val!(Display, Display);
impl_from_val!(Number, Number);
impl_from_val!(Sides, Sides);
impl_from_val!(String, String);
impl_from_val!(TextAlignment, TextAlignment);
impl_from_val!(Wrap, Wrap);

macro_rules! impl_try_from {
    ($ret:tt, $variant:ident) => {
        impl<'a> std::convert::TryFrom<&'a Value> for &'a $ret {
            type Error = ();

            fn try_from(value: &'a Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::$variant(ref a) => Ok(a),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_try_from!(Align, Alignment);
impl_try_from!(Axis, Axis);
impl_try_from!(bool, Bool);
impl_try_from!(BorderStyle, BorderStyle);
impl_try_from!(Color, Color);
impl_try_from!(Display, Display);
impl_try_from!(Number, Number);
impl_try_from!(Sides, Sides);
impl_try_from!(String, String);
impl_try_from!(TextAlignment, TextAlignment);
impl_try_from!(Wrap, Wrap);

macro_rules! try_from_int {
    ($int:ty) => {
        impl std::convert::TryFrom<&Value> for $int {
            type Error = ();

            fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Unsigned(a)) => Ok(*a as $int),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_int!(usize);
try_from_int!(u64);
try_from_int!(u32);
try_from_int!(u16);
try_from_int!(u8);

macro_rules! try_from_signed_int {
    ($int:ty) => {
        impl std::convert::TryFrom<&Value> for $int {
            type Error = ();

            fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
                match value {
                    Value::Number(Number::Signed(a)) => Ok(*a as $int),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_signed_int!(isize);
try_from_signed_int!(i64);
try_from_signed_int!(i32);
try_from_signed_int!(i16);
try_from_signed_int!(i8);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, ""),
            Self::Alignment(val) => write!(f, "{}", val),
            Self::Axis(val) => write!(f, "{:?}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::BorderStyle(val) => write!(f, "{:?}", val),
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
            Self::Sides(val) => write!(f, "{:?}", val),
            Self::String(val) => write!(f, "{}", val),
            Self::TextAlignment(val) => write!(f, "{:?}", val),
            Self::Wrap(val) => write!(f, "{:?}", val),
            Self::Transition(val, duration, easing) => {
                write!(f, "animate {val} over {duration:?} ms ({easing:?})")
            }
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
    pub fn to_data_binding(&self) -> Option<&Path> {
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

    /// The value as `Axis`
    pub fn to_axis(&self) -> Option<Axis> {
        match self {
            Self::Axis(axis) => Some(*axis),
            _ => None,
        }
    }

    /// The value as `Border`
    pub fn to_border(&self) -> Option<&BorderStyle> {
        match self {
            Self::BorderStyle(b) => Some(b),
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

    /// The value as `Wrap`
    pub fn to_wrap(&self) -> Option<Wrap> {
        match self {
            Self::Wrap(wrap) => Some(*wrap),
            _ => None,
        }
    }

    /// The value as `Sides`
    pub fn to_sides(&self) -> Option<Sides> {
        match self {
            Self::Sides(sides) => Some(*sides),
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

pub struct ValuesAttributes<'a, 'parent> {
    pub values: &'a Store<'parent>,
    attributes: &'a Attributes,
}

impl<'a, 'parent> ValuesAttributes<'a, 'parent> {
    pub fn new(values: &'a Store<'parent>, attributes: &'a Attributes) -> Self {
        let x = values.get("x");
        Self { values, attributes }
    }

    pub fn text_to_string(&self, text: &'a TextPath) -> Cow<'a, str> {
        match text {
            TextPath::Fragments(fragments) => {
                let mut output = String::new();
                for fragment in fragments {
                    match fragment {
                        Fragment::String(s) => output.push_str(s),
                        Fragment::Data(path) => {
                            let _ = path
                                .lookup_value(self.values)
                                .map(|val| write!(&mut output, "{val}"));
                        }
                    }
                }
                Cow::Owned(output)
            }
            TextPath::String(s) => Cow::from(s),
        }
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        let val = self.get_attrib(name)?;
        val.to_bool()
    }

    pub fn get_int(&self, name: &str) -> Option<u64> {
        let val = self.get_attrib(name)?;
        val.to_int()
    }

    pub fn get_signed_int(&self, name: &str) -> Option<i64> {
        let val = self.get_attrib(name)?;
        val.to_signed_int()
    }

    pub fn get_str(&self, name: &str) -> Option<Cow<'_, str>> {
        let value = self.get_attrib(name)?;
        match value {
            Value::String(s) => Some(Cow::from(s)),
            _ => None,
        }
    }

    pub fn get_attrib(&self, key: &str) -> Option<&'a Value> {
        let value = self.attributes.get(key)?;
        let Value::DataBinding(path) = value else { return Some(value) };
        path.lookup_value(self.values)
    }

    pub fn height(&self) -> Option<usize> {
        self.get_int(fields::HEIGHT).map(|i| i as usize)
    }

    pub fn width(&self) -> Option<usize> {
        self.get_int(fields::WIDTH).map(|i| i as usize)
    }

    pub fn min_width(&self) -> Option<usize> {
        self.get_int(fields::MIN_WIDTH).map(|i| i as usize)
    }

    pub fn min_height(&self) -> Option<usize> {
        self.get_int(fields::MIN_HEIGHT).map(|i| i as usize)
    }

    pub fn max_width(&self) -> Option<usize> {
        self.get_int(fields::MAX_WIDTH).map(|i| i as usize)
    }

    pub fn max_height(&self) -> Option<usize> {
        self.get_int(fields::MAX_HEIGHT).map(|i| i as usize)
    }

    pub fn factor(&self) -> Option<usize> {
        self.get_int(fields::FACTOR).map(|i| i as usize)
    }

    pub fn fill(&self) -> Option<Cow<'_, str>> {
        self.get_str(fields::FILL)
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        let value = self.get_attrib(name)?;
        match *value {
            Value::Color(color) => Some(color),
            _ => None,
        }
    }

    pub fn padding_all(&self) -> Option<Padding> {
        let left = self.padding_left();
        let right = self.padding_right();
        let top = self.padding_top();
        let bottom = self.padding_bottom();

        let padding = self.get_int(fields::PADDING).map(|p| p as usize);

        left.or_else(|| right.or_else(|| top.or_else(|| bottom.or(padding))))?;

        let padding = padding.unwrap_or(0);

        Some(Padding {
            left: left.unwrap_or(padding),
            right: right.unwrap_or(padding),
            top: top.unwrap_or(padding),
            bottom: bottom.unwrap_or(padding),
        })
    }

    pub fn padding(&self) -> Option<usize> {
        self.get_int(fields::PADDING).map(|p| p as usize)
    }

    pub fn padding_top(&self) -> Option<usize> {
        self.get_int(fields::PADDING_TOP)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_right(&self) -> Option<usize> {
        self.get_int(fields::PADDING_RIGHT)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_bottom(&self) -> Option<usize> {
        self.get_int(fields::PADDING_BOTTOM)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn padding_left(&self) -> Option<usize> {
        self.get_int(fields::PADDING_LEFT)
            .map(|i| i as usize)
            .or_else(|| self.padding())
    }

    pub fn left(&self) -> Option<i32> {
        self.get_signed_int(fields::LEFT).map(|i| i as i32)
    }

    pub fn right(&self) -> Option<i32> {
        self.get_signed_int(fields::RIGHT).map(|i| i as i32)
    }

    pub fn top(&self) -> Option<i32> {
        self.get_signed_int(fields::TOP).map(|i| i as i32)
    }

    pub fn bottom(&self) -> Option<i32> {
        self.get_signed_int(fields::BOTTOM).map(|i| i as i32)
    }

    pub fn reverse(&self) -> bool {
        self.get_bool(fields::REVERSE).unwrap_or(false)
    }

    pub fn axis(&self) -> Option<Axis> {
        match *self.get_attrib(fields::AXIS)? {
            Value::Axis(axis) => Some(axis),
            _ => None,
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        match *self.get_attrib(fields::DIRECTION)? {
            Value::Direction(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn alignment(&self) -> Option<Align> {
        match &*self.get_attrib(fields::ALIGNMENT)? {
            Value::Alignment(val) => Some(*val),
            Value::Transition(val, _, _) => match val.as_ref() {
                Value::Alignment(ref val) => Some(*val),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn max_children(&self) -> Option<usize> {
        self.get_int(fields::MAX_CHILDREN).map(|i| i as usize)
    }

    pub fn sides(&self) -> Sides {
        match self.get_attrib(fields::SIDES).as_deref() {
            Some(Value::Sides(val)) => *val,
            _ => Sides::ALL,
        }
    }

    pub fn text_alignment(&self) -> TextAlignment {
        match self.get_attrib(fields::TEXT_ALIGN).as_deref() {
            Some(Value::TextAlignment(val)) => *val,
            None | Some(_) => TextAlignment::Left,
        }
    }

    pub fn background(&self) -> Option<Color> {
        self.get_color(fields::BACKGROUND)
            .or_else(|| self.get_color(fields::BG))
    }

    pub fn foreground(&self) -> Option<Color> {
        self.get_color(fields::FOREGROUND)
            .or_else(|| self.get_color(fields::FG))
    }

    pub fn style(&self) -> Style {
        let mut inst = Style::new();
        inst.fg = self.foreground();
        inst.bg = self.background();

        if self.get_bool("bold").unwrap_or(false) {
            inst.set_bold(true);
        }

        if self.get_bool("italic").unwrap_or(false) {
            inst.set_italic(true);
        }

        if self.get_bool("dim").unwrap_or(false) {
            inst.set_dim(true);
        }

        if self.get_bool("underlined").unwrap_or(false) {
            inst.set_underlined(true);
        }

        if self.get_bool("overlined").unwrap_or(false) {
            inst.set_overlined(true);
        }

        if self.get_bool("inverse").unwrap_or(false) {
            inst.set_inverse(true);
        }

        if self.get_bool("crossed-out").unwrap_or(false) {
            inst.set_crossed_out(true);
        }

        inst
    }

    pub fn trim_start(&self) -> bool {
        self.get_bool(fields::TRIM_START).unwrap_or(true)
    }

    pub fn trim_end(&self) -> bool {
        self.get_bool(fields::TRIM_END).unwrap_or(true)
    }

    pub fn collapse_spaces(&self) -> bool {
        self.get_bool(fields::COLLAPSE_SPACES).unwrap_or(true)
    }

    pub fn auto_scroll(&self) -> bool {
        self.get_bool(fields::AUTO_SCROLL).unwrap_or(false)
    }

    pub fn word_wrap(&self) -> Wrap {
        match self.get_attrib(fields::WRAP).as_deref() {
            Some(Value::Wrap(wrap)) => *wrap,
            None | Some(_) => Wrap::Normal,
        }
    }

    pub fn display(&self) -> Display {
        match self.get_attrib(fields::DISPLAY).as_deref() {
            Some(Value::Display(display)) => *display,
            None | Some(_) => Display::Show,
        }
    }

    pub fn border_style(&self) -> Cow<'_, BorderStyle> {
        match self.get_attrib(fields::BORDER_STYLE) {
            Some(Value::BorderStyle(style)) => Cow::Borrowed(style),
            None | Some(_) => Cow::Owned(BorderStyle::Thin),
        }
    }

    pub fn id(&self) -> Option<Cow<'_, str>> {
        self.get_str(fields::ID)
    }
}
