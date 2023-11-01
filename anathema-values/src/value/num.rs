use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Num {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

impl Num {
    pub fn to_negative(self) -> Self {
        Self::Signed(-self.to_i128() as i64)
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Signed(0) | Self::Unsigned(0) => true,
            Self::Float(f) => *f == 0.0,
            _  => false
        }
    }

    pub(crate) fn to_i128(self) -> i128 {
        match self {
            Self::Signed(num) => num as i128,
            Self::Unsigned(num) => num as i128,
            Self::Float(_num) => panic!("nah, not this one"),
        }
    }

    pub fn to_usize(self) -> usize {
        match self {
            Self::Signed(num) => num as usize,
            Self::Unsigned(num) => num as usize,
            Self::Float(_num) => panic!("nah, not this one"),
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
                if lhs.abs() as u64 >= rhs {
                    Self::Signed(-((lhs.abs() as u64 - rhs) as i64))
                } else {
                    Self::Unsigned(rhs - lhs.abs() as u64)
                }
            }

            (Self::Unsigned(lhs), Self::Signed(rhs)) if rhs.is_negative() => {
                if rhs.abs() as u64 >= lhs {
                    Self::Signed(-((rhs.abs() as u64 - lhs) as i64))
                } else {
                    Self::Unsigned(lhs - rhs.abs() as u64)
                }
            }

            (Self::Signed(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs as u64 + rhs),
            (Self::Unsigned(lhs), Self::Signed(rhs)) => Self::Unsigned(rhs as u64 + lhs),
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
            _ => panic!(),
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
    };
}

macro_rules! into_signed_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Signed(n as i64)
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
