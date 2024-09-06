use std::fmt::{self, Display};
use std::ops::Deref;

use crate::{Color, Hex, Number};

/// A string that is either owned or borrowed.
/// This is not the same as Cow<T> as Cow allows the underlying value
/// to be owned and mutated.
#[derive(Debug, PartialEq)]
pub enum CommonString<'bp> {
    Owned(String),
    Borrowed(&'bp str),
}

impl<'bp> CommonString<'bp> {
    fn to_str(&self) -> &str {
        match self {
            CommonString::Owned(s) => s.as_str(),
            CommonString::Borrowed(s) => s,
        }
    }
}

impl Deref for CommonString<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.to_str()
    }
}

impl AsRef<str> for CommonString<'_> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl From<String> for CommonString<'_> {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommonVal<'frame> {
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f64),
    Hex(Hex),
    Color(Color),
    Str(&'frame str),
}

impl<'frame> CommonVal<'frame> {
    pub fn to_common_str(&self) -> CommonString<'frame> {
        match self {
            Self::Str(s) => CommonString::Borrowed(s),
            _ => self.to_string().into(),
        }
    }

    pub fn to_number(&self) -> Option<Number> {
        match self {
            Self::Int(val) => Some(Number::from(*val)),
            Self::Float(val) => Some(Number::from(*val)),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Str(s) => !s.is_empty(),
            Self::Int(i) => *i != 0,
            _ => false,
        }
    }

    pub fn to_hex(&self) -> Option<Hex> {
        match self {
            Self::Hex(hex) => Some(*hex),
            _ => None,
        }
    }

    pub fn to_color(&self) -> Option<Color> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }
}

impl Display for CommonVal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommonVal::Bool(val) => write!(f, "{val}"),
            CommonVal::Char(val) => write!(f, "{val}"),
            CommonVal::Int(val) => write!(f, "{val}"),
            CommonVal::Float(val) => write!(f, "{val:.4}"),
            CommonVal::Hex(Hex { r, g, b }) => write!(f, "r: {r}, g: {g}, b: {b}"),
            CommonVal::Color(val) => write!(f, "{val}"),
            CommonVal::Str(val) => write!(f, "{val}"),
        }
    }
}

impl From<Number> for CommonVal<'_> {
    fn from(value: Number) -> Self {
        match value.is_float() {
            true => CommonVal::Float(value.as_float()),
            false => CommonVal::Int(value.as_int()),
        }
    }
}

impl From<(u8, u8, u8)> for CommonVal<'_> {
    fn from(value: (u8, u8, u8)) -> Self {
        CommonVal::Hex(value.into())
    }
}

impl From<Hex> for CommonVal<'_> {
    fn from(value: Hex) -> Self {
        CommonVal::Hex(value)
    }
}

impl From<Color> for CommonVal<'_> {
    fn from(value: Color) -> Self {
        CommonVal::Color(value)
    }
}

impl<'a> From<&'a str> for CommonVal<'a> {
    fn from(value: &'a str) -> Self {
        CommonVal::Str(value)
    }
}

macro_rules! impl_static {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for CommonVal<'_> {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
    ($ty:ty, $variant:ident, $as:ty) => {
        impl From<$ty> for CommonVal<'_> {
            fn from(value: $ty) -> Self {
                Self::$variant(value as $as)
            }
        }
    };
}

impl_static!(bool, Bool);
impl_static!(char, Char);
impl_static!(f64, Float);
impl_static!(f32, Float, f64);

impl_static!(usize, Int, i64);
impl_static!(isize, Int, i64);
impl_static!(u64, Int, i64);
impl_static!(i64, Int, i64);
impl_static!(u32, Int, i64);
impl_static!(i32, Int, i64);
impl_static!(u16, Int, i64);
impl_static!(i16, Int, i64);
impl_static!(u8, Int, i64);
impl_static!(i8, Int, i64);

// -----------------------------------------------------------------------------
//   - Number -
// -----------------------------------------------------------------------------
impl TryFrom<CommonVal<'_>> for Number {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Int(n) => Ok(Number::I64(n)),
            CommonVal::Float(n) => Ok(Number::F64(n)),
            _ => Err(()),
        }
    }
}

impl TryFrom<&CommonVal<'_>> for Number {
    type Error = ();

    fn try_from(value: &CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Int(n) => Ok(Number::I64(*n)),
            CommonVal::Float(n) => Ok(Number::F64(*n)),
            _ => Err(()),
        }
    }
}

impl TryFrom<CommonVal<'_>> for bool {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Bool(b) => Ok(b),
            CommonVal::Int(n) => Ok(n != 0),
            CommonVal::Str(s) => Ok(!s.is_empty()),
            _ => Err(()),
        }
    }
}

macro_rules! impl_try_from_int {
    ($t:ty) => {
        impl TryFrom<CommonVal<'_>> for $t {
            type Error = ();

            fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
                match value {
                    CommonVal::Int(n) => Ok(n as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! impl_try_from_float {
    ($t:ty) => {
        impl TryFrom<CommonVal<'_>> for $t {
            type Error = ();

            fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
                match value {
                    CommonVal::Float(n) => Ok(n as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_try_from_int!(usize);
impl_try_from_int!(isize);
impl_try_from_int!(u64);
impl_try_from_int!(i64);
impl_try_from_int!(u32);
impl_try_from_int!(i32);
impl_try_from_int!(u16);
impl_try_from_int!(i16);
impl_try_from_int!(u8);
impl_try_from_int!(i8);

impl_try_from_float!(f64);
impl_try_from_float!(f32);

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::{Subscriber, Value};

    #[test]
    fn str_to_common() {
        let sub = Subscriber::ZERO;

        let value = Value::<Rc<str>>::new("hello".into());
        let value_ref = value.value_ref(sub);
        let state = value_ref.as_state().unwrap();
        let common_val = state.to_common().unwrap();
        let s = common_val.to_common_str();
        assert!(matches!(s, CommonString::Borrowed(_)));

        let value = Value::new(123u32);
        let value_ref = value.value_ref(sub);
        let state = value_ref.as_state().unwrap();
        let common_val = state.to_common().unwrap();
        let s = common_val.to_common_str();
        assert!(matches!(s, CommonString::Owned(_)));
    }

    #[test]
    fn color_to_common() {
        let sub = Subscriber::ZERO;

        let value = Value::new(Color::Rgb(36, 36, 36));
        let value_ref = value.value_ref(sub);
        let state = value_ref.as_state().unwrap();
        let common_val = state.to_common().unwrap();
        let s = common_val.to_common_str();
        assert!(matches!(s, CommonString::Owned(_)));

        let value = CommonVal::Color(Color::Grey);
        let color = value.to_color().unwrap();
        assert_eq!(color, Color::Grey);
    }
}
