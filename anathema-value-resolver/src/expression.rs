use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use anathema_state::{Hex, Number, PendingValue, Subscriber, Type, ValueRef};
use anathema_strings::StrIndex;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::{Expression, Primitive};

use crate::value::ValueKind;

macro_rules! or_null {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => return ValueExpr::Null,
        }
    };
}

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
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
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

impl<'bp> From<Primitive> for ValueExpr<'bp> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(b) => Self::Bool(Kind::Static(b)),
            Primitive::Char(c) => Self::Char(Kind::Static(c)),
            Primitive::Int(i) => Self::Int(Kind::Static(i)),
            Primitive::Float(f) => Self::Float(Kind::Static(f)),
            Primitive::Hex(hex) => Self::Hex(Kind::Static(hex)),
        }
    }
}

// Resolve an expression to a value kind, this is the final value in the chain
pub(crate) fn resolve_value<'bp>(expr: &ValueExpr<'bp>, sub: Subscriber) -> ValueKind<'bp> {
    match expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
        ValueExpr::Bool(Kind::Static(b)) => ValueKind::Bool(*b),
        ValueExpr::Bool(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Bool(state.as_bool().unwrap())
        }
        ValueExpr::Char(Kind::Static(c)) => ValueKind::Char(*c),
        ValueExpr::Char(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Char(state.as_char().unwrap())
        }
        ValueExpr::Int(Kind::Static(i)) => ValueKind::Int(*i),
        ValueExpr::Int(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Int(state.as_int().unwrap())
        }
        ValueExpr::Float(Kind::Static(f)) => ValueKind::Float(*f),
        ValueExpr::Float(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Float(state.as_float().unwrap())
        }
        ValueExpr::Hex(Kind::Static(h)) => ValueKind::Hex(*h),
        ValueExpr::Hex(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Hex(state.as_hex().unwrap())
        }
        ValueExpr::Str(Kind::Static(s)) => ValueKind::Str(Cow::Borrowed(s)),
        ValueExpr::Str(Kind::Dyn(val)) => {
            let state = val.as_state().unwrap();
            let s = state.as_str().unwrap();
            ValueKind::Str(Cow::Owned(s.to_owned()))
        }

        // -----------------------------------------------------------------------------
        //   - Operations and conditionals -
        // -----------------------------------------------------------------------------
        ValueExpr::Not(value_expr) => {
            let ValueKind::Bool(val) = resolve_value(value_expr, sub) else { return ValueKind::Null };
            ValueKind::Bool(!val)
        }
        ValueExpr::Negative(value_expr) => match resolve_value(value_expr, sub) {
            ValueKind::Int(n) => ValueKind::Int(-n),
            ValueKind::Float(n) => ValueKind::Float(-n),
            _ => ValueKind::Null,
        },
        ValueExpr::Equality(lhs, rhs, equality) => {
            let lhs = resolve_value(lhs, sub);
            let rhs = resolve_value(rhs, sub);
            let b = match equality {
                Equality::Eq => lhs == rhs,
                Equality::NotEq => lhs != rhs,
                Equality::Gt => lhs > rhs,
                Equality::Gte => lhs >= rhs,
                Equality::Lt => lhs < rhs,
                Equality::Lte => lhs <= rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::LogicalOp(lhs, rhs, logical_op) => {
            let ValueKind::Bool(lhs) = resolve_value(lhs, sub) else { return ValueKind::Null };
            let ValueKind::Bool(rhs) = resolve_value(rhs, sub) else { return ValueKind::Null };
            let b = match logical_op {
                LogicalOp::And => lhs && rhs,
                LogicalOp::Or => lhs || rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::Op(lhs, rhs, op) => match (resolve_value(lhs, sub), resolve_value(rhs, sub)) {
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
            _ => ValueKind::Null,
        },
        ValueExpr::Either(first, second) => match resolve_value(first, sub) {
            ValueKind::Null => resolve_value(second, sub),
            first => first,
        },

        // -----------------------------------------------------------------------------
        //   - Maps and lists -
        // -----------------------------------------------------------------------------
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
            resolve_value(&expr, sub)
        }

        // -----------------------------------------------------------------------------
        //   - Call -
        // -----------------------------------------------------------------------------
        ValueExpr::Call => todo!(),

        // -----------------------------------------------------------------------------
        //   - Null -
        // -----------------------------------------------------------------------------
        ValueExpr::Null => ValueKind::Null,
    }
}

fn resolve_pending(val: PendingValue, sub: Subscriber) -> ValueExpr<'static> {
    val.subscribe(sub);
    match val.type_info() {
        Type::Int => ValueExpr::Int(Kind::Dyn(val)),
        Type::Float => ValueExpr::Float(Kind::Dyn(val)),
        Type::Char => ValueExpr::Char(Kind::Dyn(val)),
        Type::String => ValueExpr::Str(Kind::Dyn(val)),
        Type::Bool => ValueExpr::Bool(Kind::Dyn(val)),
        Type::Map => ValueExpr::DynMap(val),
        Type::List => ValueExpr::DynList(val),
        // Type::Composite => ValueKind::Composite,
        val_type => panic!("{val_type:?}"),
    }
}

fn resolve_index<'bp>(src: &ValueExpr<'bp>, index: &ValueExpr<'bp>, sub: Subscriber) -> ValueExpr<'bp> {
    match src {
        ValueExpr::DynMap(value) => {
            let s = or_null!(value.as_state());
            let map = s.as_any_map().expect("a dyn map is always an any_map");
            let key = or_null!(resolve_str(index, sub));
            let val = or_null!(map.lookup(key.to_str()));
            resolve_pending(val, sub)
        }
        ValueExpr::DynList(value) => {
            let s = or_null!(value.as_state());
            let list = s.as_any_list().expect("a dyn list is always an any_list");
            let key = resolve_int(index, sub);
            let val = or_null!(list.lookup(key.to_int() as usize));
            resolve_pending(val, sub)
        }
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => {
            let key = or_null!(resolve_str(index, sub));
            or_null!(hash_map.get(&*key.to_str()).cloned())
        }
        ValueExpr::Index(inner_src, inner_index) => {
            let src = resolve_index(inner_src, inner_index, sub);
            resolve_index(&src, index, sub)
        }
        ValueExpr::Either(first, second) => {
            let src = match resolve_expr(first, sub) {
                None | Some(ValueExpr::Null) => match resolve_expr(second, sub) {
                    None | Some(ValueExpr::Null) => return ValueExpr::Null,
                    Some(e) => e,
                },
                Some(e) => e,
            };
            resolve_index(&src, index, sub)
        }
        ValueExpr::Null => ValueExpr::Null,
        _ => unreachable!(),
    }
}

fn resolve_expr<'a, 'bp>(expr: &'a ValueExpr<'bp>, sub: Subscriber) -> Option<ValueExpr<'bp>> {
    match expr {
        ValueExpr::Either(first, second) => match resolve_expr(first, sub) {
            None | Some(ValueExpr::Null) => resolve_expr(second, sub),
            expr => expr,
        },
        ValueExpr::Index(src, index) => Some(resolve_index(src, index, sub)),
        _ => None,
        // ValueExpr::Bool(_) |
        // ValueExpr::Char(_) |
        // ValueExpr::Int(_) |
        // ValueExpr::Float(_) |
        // ValueExpr::Hex(_) |
        // ValueExpr::Str(_) => expr,
        // _ => panic!(),
    }
}

fn resolve_str<'a, 'bp>(index: &'a ValueExpr<'bp>, sub: Subscriber) -> Option<Kind<&'bp str>> {
    match index {
        ValueExpr::Str(kind) => Some(*kind),
        ValueExpr::Index(src, index) => match resolve_index(src, index, sub) {
            ValueExpr::Str(kind) => Some(kind),
            _ => None,
        },
        ValueExpr::Either(first, second) => resolve_str(first, sub).or_else(|| resolve_str(second, sub)),
        ValueExpr::Null => None,
        ValueExpr::Call => todo!(),
        _ => None,
    }
}

fn resolve_int<'a, 'bp>(index: &'a ValueExpr<'bp>, sub: Subscriber) -> Kind<i64> {
    match index {
        ValueExpr::Int(kind) => *kind,
        _ => panic!(),
    }
}

fn int_op(lhs: i64, rhs: i64, op: Op) -> i64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}

fn float_op(lhs: f64, rhs: f64, op: Op) -> f64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}
