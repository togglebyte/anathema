pub use anathema_state_derive::State;
use anathema_store::slab::Key;

pub use crate::colors::{Color, FromColor};
pub use crate::common::{CommonString, CommonVal};
pub use crate::numbers::Number;
pub use crate::states::{AnyState, State, StateId, States};
pub use crate::store::{
    clear_all_changes, clear_all_futures, clear_all_subs, debug, drain_changes, drain_futures, register_future, Change,
    Changes, FutureValues, Subscriber,
};
pub use crate::value::{List, Map, PendingValue, SharedState, Value, ValueRef};

mod colors;
mod common;
mod numbers;
mod states;
mod store;
mod value;

// -----------------------------------------------------------------------------
//   - Macro requirements -
// -----------------------------------------------------------------------------
#[allow(unused_extern_crates)]
extern crate self as anathema;
#[allow(unused_imports)]
pub use crate as state;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Path<'e> {
    Key(&'e str),
    Index(usize),
}

impl From<usize> for Path<'_> {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Self {
        Self::Key(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hex {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Hex {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
}

impl From<(u8, u8, u8)> for Hex {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl TryFrom<&str> for Hex {
    type Error = ();

    fn try_from(hex: &str) -> Result<Self, Self::Error> {
        if hex.is_empty() || !hex.starts_with("#") {
            return Err(());
        }

        let hex = &hex[1..];
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| ())?;
                let r = r << 4 | r;
                let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| ())?;
                let g = g << 4 | g;
                let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| ())?;
                let b = b << 4 | b;
                Ok(Self::from((r, g, b)))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
                Ok(Self::from((r, g, b)))
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str_for_hex() {
        let case1 = "#aabbcc";
        let case2 = "#abc";

        let result_1 = Hex::try_from(case1).unwrap();
        let result_2 = Hex::try_from(case2).unwrap();
        let expected = Hex::from((0xaa, 0xbb, 0xcc));

        assert_eq!(result_1, expected);
        assert_eq!(result_2, expected);
    }
}
