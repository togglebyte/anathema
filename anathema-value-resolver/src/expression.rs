use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use anathema_state::{Hex, Number, PendingValue, Subscriber, Type, ValueRef};
use anathema_strings::StrIndex;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::{Expression, Primitive};

use crate::value::ValueKind;

#[derive(Debug)]
pub enum Str<'bp> {
    Borrowed(&'bp str),
    Owned(ValueRef, String),
}

#[derive(Debug, Copy, Clone)]
pub enum Kind<T> {
    Static(T),
    Dyn(PendingValue),
}

impl<'bp> Kind<&'bp str> {
    pub(crate) fn to_str(&self) -> Cow<'bp, str> {
        match self {
            Kind::Static(s) => Cow::Borrowed(s),
            Kind::Dyn(pending_value) => pending_value
                .as_state()
                .map(|s| s.as_str().unwrap().to_owned())
                .unwrap()
                .into(),
        }
    }
}

impl Kind<i64> {
    pub(crate) fn to_int(&self) -> i64 {
        match self {
            Kind::Static(s) => *s,
            Kind::Dyn(pending_value) => pending_value
                .as_state()
                .map(|s| s.as_int().unwrap().to_owned())
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueExpr<'bp> {
    Bool(bool),
    Char(char),
    Int(Kind<i64>),
    Float(f64),
    Hex(Hex),
    Str(Kind<&'bp str>),
    DynMap(PendingValue),
    DynList(PendingValue),
    List(Box<[Self]>),
    Map(HashMap<&'bp str, Self>),
    Index(Box<Self>, Box<Self>),

    Not(Box<Self>),
    Negative(Box<Self>),

    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    Op(Box<Self>, Box<Self>, Op),
    Either(Box<Self>, Box<Self>),

    // Dyn(ValueRef),
    Call,

    Null,
}

impl<'bp> PartialEq for ValueExpr<'bp> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'bp> PartialOrd for ValueExpr<'bp> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl<'bp> From<PendingValue> for ValueExpr<'bp> {
    fn from(value: PendingValue) -> Self {
        match value.type_info() {
            Type::Int => Self::Int(Kind::Dyn(value)),
            Type::Float => todo!(),
            Type::Char => todo!(),
            Type::String => Self::Str(Kind::Dyn(value)),
            Type::Bool => todo!(),
            Type::Map => Self::DynMap(value),
            Type::List => Self::DynList(value),
            Type::Composite => todo!(),
        }
    }
}

impl<'bp> From<Primitive> for ValueExpr<'bp> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(b) => Self::Bool(b),
            Primitive::Char(c) => Self::Char(c),
            Primitive::Int(i) => Self::Int(Kind::Static(i)),
            Primitive::Float(f) => Self::Float(f),
            Primitive::Hex(hex) => Self::Hex(hex),
        }
    }
}

// impl From<Number> for Value<'_> {
//     fn from(number: Number) -> Self {
//         match number {
//             Number::I64(i) => Value::Int(i),
//             Number::U64(i) => Value::Int(i as i64),
//             Number::Usize(i) => Value::Int(i as i64),
//             Number::Isize(i) => Value::Int(i as i64),
//             Number::U32(i) => Value::Int(i as i64),
//             Number::I32(i) => Value::Int(i as i64),
//             Number::U16(i) => Value::Int(i as i64),
//             Number::I16(i) => Value::Int(i as i64),
//             Number::U8(i) => Value::Int(i as i64),
//             Number::I8(i) => Value::Int(i as i64),
//             Number::F64(f) => Value::Float(f),
//             Number::F32(f) => Value::Float(f as f64),
//         }
//     }
// }

impl<'bp> ValueExpr<'bp> {
    // fn stringify(&self, strings: &HStrings<'_>) {}
}

trait ValueExprVisitor {}

struct IndexVisitor {}

pub(crate) fn resolve_expr<'bp>(expr: &ValueExpr<'bp>, sub: Subscriber) -> ValueKind<'bp> {
    match expr {
        ValueExpr::Bool(_) => todo!(),
        ValueExpr::Char(_) => todo!(),
        ValueExpr::Int(kind) => match kind {
            Kind::Static(i) => ValueKind::Int(*i),
            Kind::Dyn(pending_value) => {
                pending_value.subscribe(sub);
                let state = pending_value.as_state().unwrap();
                ValueKind::Int(state.as_int().unwrap())
            }
        },
        ValueExpr::Float(_) => todo!(),
        ValueExpr::Hex(hex) => todo!(),
        ValueExpr::Str(Kind::Static(s)) => ValueKind::String(Cow::Borrowed(s)),
        ValueExpr::Str(Kind::Dyn(val)) => {
            let state = val.as_state().unwrap();
            let s = state.as_str().unwrap();
            ValueKind::String(Cow::Owned(s.to_owned()))
        }
        ValueExpr::DynMap(value) => {
            let state = value.as_state().unwrap();
            state.as_any_map();
            panic!()
        }
        ValueExpr::DynList(pending_value) => todo!(),
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => todo!(),
        ValueExpr::Index(src, index) => {
            let expr = resolve_index(src, index, sub);
            resolve_expr(&expr, sub)
        }
        ValueExpr::Not(value_expr) => todo!(),
        ValueExpr::Negative(value_expr) => todo!(),
        ValueExpr::Equality(value_expr, value_expr1, equality) => todo!(),
        ValueExpr::LogicalOp(value_expr, value_expr1, logical_op) => todo!(),
        ValueExpr::Op(value_expr, value_expr1, op) => todo!(),
        ValueExpr::Either(value_expr, value_expr1) => todo!(),
        ValueExpr::Call => todo!(),
        ValueExpr::Null => todo!(),
    }
}

fn resolve_index<'bp>(src: &ValueExpr<'bp>, index: &ValueExpr<'bp>, sub: Subscriber) -> ValueExpr<'bp> {
    match src {
        ValueExpr::DynMap(value) => {
            let s = value.as_state().unwrap();
            let map = s.as_any_map().expect("a dyn map is always a any_map");
            let key = resolve_str(index, sub);
            let val = map.lookup(key.to_str()).unwrap();
            match val.type_info() {
                Type::Int => ValueExpr::Int(Kind::Dyn(val)),
                // Type::Float => ValueKind::Float,
                // Type::Char => ValueKind::Char,
                Type::String => ValueExpr::Str(Kind::Dyn(val)),
                // Type::Bool => ValueKind::Bool,
                Type::Map => ValueExpr::DynMap(val),
                // Type::List => ValueKind::List,
                // Type::Composite => ValueKind::Composite,
                _ => panic!(),
            }
        }
        ValueExpr::DynList(pending_value) => todo!(),
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => {
            let key = resolve_str(index, sub);
            hash_map.get(&*key.to_str()).cloned().unwrap()
        }
        ValueExpr::Index(inner_src, inner_index) => {
            let src = resolve_index(inner_src, inner_index, sub);
            resolve_index(&src, index, sub)
        }
        ValueExpr::Either(value_expr, value_expr1) => todo!(),
        _ => unreachable!(),
    }
}

fn resolve_str<'a, 'bp>(index: &'a ValueExpr<'bp>, sub: Subscriber) -> Kind<&'bp str> {
    match index {
        ValueExpr::Str(kind) => *kind,
        ValueExpr::Index(src, index) => {
            match resolve_index(src, index, sub) {
                ValueExpr::Str(kind) => kind,
                _ => unreachable!(),
            }
            
        }
        ValueExpr::Either(value_expr, value_expr1) => todo!(),
        ValueExpr::Call => todo!(),
        ValueExpr::Null => todo!(),

        ValueExpr::Bool(_) => todo!(),
        ValueExpr::Char(_) => todo!(),
        ValueExpr::Int(kind) => todo!(),
        ValueExpr::Float(_) => todo!(),
        ValueExpr::Hex(hex) => todo!(),
        ValueExpr::DynMap(pending_value) => todo!(),
        ValueExpr::DynList(pending_value) => todo!(),
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => todo!(),
        ValueExpr::Not(value_expr) => todo!(),
        ValueExpr::Negative(value_expr) => todo!(),
        ValueExpr::Equality(value_expr, value_expr1, equality) => todo!(),
        ValueExpr::LogicalOp(value_expr, value_expr1, logical_op) => todo!(),
        ValueExpr::Op(value_expr, value_expr1, op) => todo!(),
    }
}
