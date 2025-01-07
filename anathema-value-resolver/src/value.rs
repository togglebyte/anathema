use std::collections::HashMap;
use std::ops::Deref;

use anathema_state::{Hex, Number, PendingValue, ValueRef};
use anathema_strings::StrIndex;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::Primitive;

#[derive(Debug)]
pub enum Str<'bp> {
    Borrowed(&'bp str),
    Owned(ValueRef, String),
}

#[derive(Debug)]
pub enum Kind<T> {
    Static(T),
    Dyn { source: PendingValue, cache: T },
}

impl<T> Deref for Kind<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Kind::Static(val) => val,
            Kind::Dyn { cache, .. } => cache,
        }
    }
}

#[derive(Debug)]
pub enum Value<'bp> {
    Number(Kind<Number>),
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f64),
    Hex(Hex),
    Str(&'bp str),
    List(Box<[Self]>),
    Map(HashMap<&'bp str, Self>),
    Index(Box<Self>, Box<Self>),

    Not(Box<Self>),
    Negative(Box<Self>),

    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    Op(Box<Self>, Box<Self>, Op),
    Either(Box<Self>, Box<Self>),

    Dyn(ValueRef),
    Call,

    Null,
}

impl<'bp> Value<'bp> {}

impl<'bp> PartialEq for Value<'bp> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'bp> PartialOrd for Value<'bp> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl<'bp> From<Primitive> for Value<'bp> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(b) => Self::Bool(b),
            Primitive::Char(c) => Self::Char(c),
            Primitive::Int(i) => Self::Int(i),
            Primitive::Float(f) => Self::Float(f),
            Primitive::Hex(hex) => Self::Hex(hex),
        }
    }
}

impl From<Number> for Value<'_> {
    fn from(number: Number) -> Self {
        match number {
            Number::I64(i) => Value::Int(i),
            Number::U64(i) => Value::Int(i as i64),
            Number::Usize(i) => Value::Int(i as i64),
            Number::Isize(i) => Value::Int(i as i64),
            Number::U32(i) => Value::Int(i as i64),
            Number::I32(i) => Value::Int(i as i64),
            Number::U16(i) => Value::Int(i as i64),
            Number::I16(i) => Value::Int(i as i64),
            Number::U8(i) => Value::Int(i as i64),
            Number::I8(i) => Value::Int(i as i64),
            Number::F64(f) => Value::Float(f),
            Number::F32(f) => Value::Float(f as f64),
        }
    }
}

impl<'bp> Value<'bp> {
    // fn stringify(&self, strings: &HStrings<'_>) {}
}
