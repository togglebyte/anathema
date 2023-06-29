use anathema_render::Color;

use crate::{Number, Value, Align, Axis, BorderStyle, Display, Sides, TextAlignment, Wrap};

pub trait ValueTryIntoRef<'a, T> {
    fn try_into_type_ref(&self) -> Option<&T>;
}

pub trait ValueTryIntoMut<'a, T> {
    fn try_into_type_mut(&mut self) -> Option<&mut T>;
}

macro_rules! value_into {
    ($t:ty, $variant:ident) => {
        impl<'a> ValueTryIntoMut<'a, $t> for Value {
            fn try_into_type_mut(&mut self) -> Option<&mut $t> {
                match self {
                    Value::$variant(ref mut v) => Some(v),
                    _ => None
                }
            }
        }

        impl<'a> ValueTryIntoRef<'a, $t> for Value {
            fn try_into_type_ref(&self) -> Option<&$t> {
                match self {
                    Value::$variant(ref v) => Some(v),
                    _ => None
                }
            }
        }
    };
}

macro_rules! value_into_num {
    ($t:ty, $variant:ident) => {
        impl<'a> ValueTryIntoMut<'a, $t> for Value {
            fn try_into_type_mut(&mut self) -> Option<&mut $t> {
                match self {
                    Value::Number(Number::$variant(ref mut v)) => Some(v),
                    _ => None
                }
            }
        }

        impl<'a> ValueTryIntoRef<'a, $t> for Value {
            fn try_into_type_ref(&self) -> Option<&$t> {
                match self {
                    Value::Number(Number::$variant(ref v)) => Some(v),
                    _ => None
                }
            }
        }
    };
}

value_into!(Align, Alignment);
value_into!(Axis, Axis);
value_into!(bool, Bool);
value_into!(BorderStyle, BorderStyle);
value_into!(Color, Color);
value_into!(Display, Display);
value_into!(Number, Number);
value_into!(Sides, Sides);
value_into!(String, String);
value_into!(TextAlignment, TextAlignment);
value_into!(Wrap, Wrap);

value_into_num!(i64, Signed);
value_into_num!(u64, Unsigned);
value_into_num!(f64, Float);
