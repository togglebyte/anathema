use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};

macro_rules! to_num {
    ($fn_name:ident, $num_type:ty) => {
        pub fn $fn_name(self) -> $num_type {
            match self {
                Self::Signed(num) => num as $num_type,
                Self::Unsigned(num) => num as $num_type,
                Self::Float(num) => num as $num_type,
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Num {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

impl Num {
    to_num!(to_f64, f64);

    to_num!(to_f32, f32);

    to_num!(to_i128, i128);

    to_num!(to_u128, u128);

    to_num!(to_usize, usize);

    to_num!(to_isize, isize);

    to_num!(to_u64, u64);

    to_num!(to_i64, i64);

    to_num!(to_u32, u32);

    to_num!(to_i32, i32);

    to_num!(to_u16, u16);

    to_num!(to_i16, i16);

    to_num!(to_u8, u8);

    to_num!(to_i8, i8);

    pub fn to_negative(self) -> Self {
        Self::Signed(-self.to_i128() as i64)
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Signed(0) | Self::Unsigned(0) => true,
            Self::Float(f) => *f == 0.0,
            _ => false,
        }
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed(n) => write!(f, "{n}"),
            Self::Unsigned(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
        }
    }
}

impl Mul for Num {
    type Output = Num;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs * rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs * rhs),
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs * rhs),
            _ => panic!(),
        }
    }
}

impl Add for Num {
    type Output = Num;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs + rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs + rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) if lhs.is_negative() => {
                if lhs.unsigned_abs() >= rhs {
                    Self::Signed(-((lhs.unsigned_abs() - rhs) as i64))
                } else {
                    Self::Unsigned(rhs - lhs.unsigned_abs())
                }
            }

            (Self::Unsigned(lhs), Self::Signed(rhs)) if rhs.is_negative() => {
                if rhs.unsigned_abs() >= lhs {
                    Self::Signed(-((rhs.unsigned_abs() - lhs) as i64))
                } else {
                    Self::Unsigned(lhs - rhs.unsigned_abs())
                }
            }

            (Self::Signed(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs as u64 + rhs),
            (Self::Unsigned(lhs), Self::Signed(rhs)) => Self::Unsigned(rhs as u64 + lhs),
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs + rhs),
            _ => panic!(),
        }
    }
}

impl Sub for Num {
    type Output = Num;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs - rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs - rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs - rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs - rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs - rhs),
            _ => panic!(),
        }
    }
}

impl Div for Num {
    type Output = Num;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs / rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs / rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs / rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs / rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs / rhs),
            _ => todo!(),
        }
    }
}

impl Rem for Num {
    type Output = Num;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs % rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs % rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs % rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs % rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs % rhs),
            _ => panic!(),
        }
    }
}

macro_rules! into_unsigned_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Unsigned(n as u64)
            }
        }

        impl From<&$t> for Num {
            fn from(n: &$t) -> Self {
                Self::Unsigned(*n as u64)
            }
        }
    };
}

macro_rules! into_signed_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Signed(n as i64)
            }
        }

        impl From<&$t> for Num {
            fn from(n: &$t) -> Self {
                Self::Signed(*n as i64)
            }
        }
    };
}

macro_rules! into_float_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Float(n as f64)
            }
        }

        impl From<&$t> for Num {
            fn from(n: &$t) -> Self {
                Self::Float(*n as f64)
            }
        }
    };
}

into_unsigned_num!(u8);
into_unsigned_num!(u16);
into_unsigned_num!(u32);
into_unsigned_num!(u64);
into_unsigned_num!(usize);

into_signed_num!(i8);
into_signed_num!(i16);
into_signed_num!(i32);
into_signed_num!(i64);
into_signed_num!(isize);

into_float_num!(f32);
into_float_num!(f64);
