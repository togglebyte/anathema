use std::any::Any;
use std::fmt::Debug;

use anathema_store::slab::{Slab, SlabIndex};

use crate::value::{AnonValue, SubKey, Type};
use crate::{Color, Hex, Value};

pub trait TypeId {
    const TYPE: Type = Type::Composite;
}

impl TypeId for char {
    const TYPE: Type = Type::Char;
}

impl TypeId for String {
    const TYPE: Type = Type::String;
}

impl TypeId for bool {
    const TYPE: Type = Type::Bool;
}

impl TypeId for Hex {
    const TYPE: Type = Type::Hex;
}

impl<K, V> TypeId for crate::Map<K, V> {
    const TYPE: Type = Type::Map;
}

impl<K, V> TypeId for crate::List<K, V> {
    const TYPE: Type = Type::List;
}

impl TypeId for () {
    const TYPE: Type = Type::Unit;
}

impl TypeId for Color {
    const TYPE: Type = Type::Color;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StateId(usize);

impl StateId {
    pub const ZERO: Self = Self(0);
}

impl SlabIndex for StateId {
    const MAX: usize = usize::MAX;

    fn as_usize(&self) -> usize {
        self.0
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index)
    }
}

pub trait State<K: SubKey>: Any + 'static {
    fn type_info(&self) -> Type;

    fn as_int(&self) -> Option<i64> {
        None
    }

    fn as_float(&self) -> Option<f64> {
        None
    }

    fn as_hex(&self) -> Option<Hex> {
        None
    }

    fn as_color(&self) -> Option<Color> {
        None
    }

    fn as_char(&self) -> Option<char> {
        None
    }

    fn as_str(&self) -> Option<&str> {
        None
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap<K>> {
        None
    }

    fn as_any_list(&self) -> Option<&dyn AnyList<K>> {
        None
    }

    fn as_maybe(&self) -> Option<&dyn AnyMaybe<K>> {
        None
    }
}

impl<K: SubKey> Debug for dyn State<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<State ({:?})>", self.type_info())
    }
}

impl<K: SubKey> State<K> for Box<dyn State<K>> {
    fn type_info(&self) -> Type {
        self.as_ref().type_info()
    }

    fn as_int(&self) -> Option<i64> {
        self.as_ref().as_int()
    }

    fn as_float(&self) -> Option<f64> {
        self.as_ref().as_float()
    }

    fn as_char(&self) -> Option<char> {
        self.as_ref().as_char()
    }

    fn as_hex(&self) -> Option<Hex> {
        self.as_ref().as_hex()
    }

    fn as_color(&self) -> Option<Color> {
        self.as_ref().as_color()
    }

    fn as_str(&self) -> Option<&str> {
        self.as_ref().as_str()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_ref().as_bool()
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap<K>> {
        self.as_ref().as_any_map()
    }

    fn as_any_list(&self) -> Option<&dyn AnyList<K>> {
        self.as_ref().as_any_list()
    }

    fn as_maybe(&self) -> Option<&dyn AnyMaybe<K>> {
        self.as_ref().as_maybe()
    }
}

pub trait AnyMap<K: SubKey>: 'static {
    fn lookup(&self, key: &str) -> Option<AnonValue<K>>;

    fn is_empty(&self) -> bool;
}

pub trait AnyList<K: SubKey>: 'static {
    fn lookup(&self, index: usize) -> Option<AnonValue<K>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: SubKey> dyn AnyList<K> {
    pub fn iter(&self) -> impl Iterator<Item = AnonValue<K>> {
        let len = self.len();
        (0..len).filter_map(|i| self.lookup(i))
    }
}

pub trait AnyMaybe<K: SubKey> {
    fn get(&self) -> Option<AnonValue<K>>;
}

macro_rules! impl_num_state {
    ($t:ty) => {
        impl TypeId for $t {
            const TYPE: Type = Type::Int;
        }

        impl<K: SubKey> State<K> for $t {
            fn type_info(&self) -> Type {
                Type::Int
            }

            fn as_int(&self) -> Option<i64> {
                Some(*self as i64)
            }
        }
    };
}

macro_rules! impl_float_state {
    ($t:ty) => {
        impl TypeId for $t {
            const TYPE: Type = Type::Float;
        }

        impl<K: SubKey> State<K> for $t {
            fn type_info(&self) -> Type {
                Type::Float
            }

            fn as_float(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }
    };
}

impl<K: SubKey> State<K> for bool {
    fn type_info(&self) -> Type {
        Type::Bool
    }

    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }
}

impl<K: SubKey> State<K> for String {
    fn type_info(&self) -> Type {
        Type::String
    }

    fn as_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl<K: SubKey> State<K> for &'static str {
    fn type_info(&self) -> Type {
        Type::String
    }

    fn as_str(&self) -> Option<&str> {
        Some(*self)
    }
}

impl<K: SubKey> State<K> for char {
    fn type_info(&self) -> Type {
        Type::Char
    }

    fn as_char(&self) -> Option<char> {
        Some(*self)
    }
}

impl<K: SubKey> State<K> for Hex {
    fn type_info(&self) -> Type {
        Type::Hex
    }

    fn as_hex(&self) -> Option<Hex> {
        Some(*self)
    }
}

impl<K: SubKey> State<K> for Color {
    fn type_info(&self) -> Type {
        Type::Color
    }

    fn as_color(&self) -> Option<Color> {
        Some(*self)
    }
}

impl<K: SubKey> State<K> for () {
    fn type_info(&self) -> Type {
        Type::Unit
    }
}

impl_num_state!(u8);
impl_num_state!(i8);
impl_num_state!(u16);
impl_num_state!(i16);
impl_num_state!(u32);
impl_num_state!(i32);
impl_num_state!(u64);
impl_num_state!(i64);
impl_num_state!(usize);
impl_num_state!(isize);
impl_float_state!(f32);
impl_float_state!(f64);
