//! A general numerical type used to cast between template values
//! and state values.
//!
//! Supports general maths operations.
//!
//! Note: If either the `lhs` or the `rhs` is a float then the entire
//! number has to be treated as a float
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::{CommonVal, State};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Number {
    Usize(usize),
    Isize(isize),
    U64(u64),
    I64(i64),
    U32(u32),
    I32(i32),
    U16(u16),
    I16(i16),
    U8(u8),
    I8(i8),
    F64(f64),
    F32(f32),
}

impl Number {
    pub fn as_float(self) -> f64 {
        match self {
            Self::Usize(n) => n as f64,
            Self::Isize(n) => n as f64,
            Self::U64(n) => n as f64,
            Self::I64(n) => n as f64,
            Self::U32(n) => n as f64,
            Self::I32(n) => n as f64,
            Self::U16(n) => n as f64,
            Self::I16(n) => n as f64,
            Self::U8(n) => n as f64,
            Self::I8(n) => n as f64,
            Self::F64(n) => n,
            Self::F32(n) => n as f64,
        }
    }

    pub fn as_int(self) -> i64 {
        match self {
            Self::Usize(n) => n as i64,
            Self::Isize(n) => n as i64,
            Self::U64(n) => n as i64,
            Self::I64(n) => n,
            Self::U32(n) => n as i64,
            Self::I32(n) => n as i64,
            Self::U16(n) => n as i64,
            Self::I16(n) => n as i64,
            Self::U8(n) => n as i64,
            Self::I8(n) => n as i64,
            Self::F64(n) => n as i64,
            Self::F32(n) => n as i64,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::F64(_) | Self::F32(_))
    }

    pub fn as_uint(self) -> usize {
        match self {
            Self::Usize(n) => n,
            Self::Isize(n @ 0..isize::MAX) => n as usize,
            Self::U64(n) => n as usize,
            Self::I64(n @ 0..=i64::MAX) => n as usize,
            Self::U32(n) => n as usize,
            Self::I32(n @ 0..=i32::MAX) => n as usize,
            Self::U16(n) => n as usize,
            Self::I16(n @ 0..=i16::MAX) => n as usize,
            Self::U8(n) => n as usize,
            Self::I8(n @ 0..=i8::MAX) => n as usize,
            Self::F64(n @ 0.0..=f64::MAX) => n as usize,
            Self::F32(n @ 0.0..=f32::MAX) => n as usize,
            _ => 0,
        }
    }
}

impl State for Number {
    fn to_number(&self) -> Option<Number> {
        Some(*self)
    }

    fn to_bool(&self) -> bool {
        self.as_int() != 0
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        let val = match self.is_float() {
            true => CommonVal::Float(self.as_float()),
            false => CommonVal::Int(self.as_int()),
        };
        Some(val)
    }
}

struct IsFloat(Number, Number);

impl IsFloat {
    fn is_float(&self) -> bool {
        self.0.is_float() || self.1.is_float()
    }
}

impl From<(Number, Number)> for IsFloat {
    fn from((a, b): (Number, Number)) -> Self {
        IsFloat(a, b)
    }
}

macro_rules! impl_from {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for Number {
            fn from(val: $ty) -> Self {
                Self::$variant(val)
            }
        }
    };
}

impl_from!(usize, Usize);
impl_from!(isize, Isize);
impl_from!(u64, U64);
impl_from!(i64, I64);
impl_from!(u32, U32);
impl_from!(i32, I32);
impl_from!(u16, U16);
impl_from!(i16, I16);
impl_from!(u8, U8);
impl_from!(i8, I8);
impl_from!(f64, F64);
impl_from!(f32, F32);

macro_rules! impl_op {
    ($trait:ty, $fn:ident, $op:tt) => {
        impl $trait for Number {
            type Output = Number;

            fn $fn(self, rhs: Self) -> Self::Output {
                match IsFloat(self, rhs).is_float() {
                    true => Number::F64(self.as_float() $op rhs.as_float()),
                    false => Number::I64(self.as_int() $op rhs.as_int()),
                }
            }
        }
    }
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_op!(Rem, rem, %);

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output {
        match self.is_float() {
            true => Number::F64(-self.as_float()),
            false => Number::I64(-self.as_int()),
        }
    }
}
