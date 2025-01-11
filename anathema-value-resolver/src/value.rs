use std::borrow::Cow;

use anathema_state::{Hex, Subscriber};

use crate::expression::{resolve_expr, ValueExpr};

/// This is the final value for a node attribute / value.
/// This should be evaluated fully for the `ValueKind`
#[derive(Debug)]
pub struct Value<'bp> {
    expr: ValueExpr<'bp>,
    sub: Subscriber,
    kind: ValueKind<'bp>,
}

impl<'bp> Value<'bp> {
    pub fn new(expr: ValueExpr<'bp>, sub: Subscriber) -> Self {
        let kind = resolve_expr(&expr, sub);
        Self {
            expr,
            sub,
            kind, 
        }
    }

    pub fn reload(&mut self) {
        self.kind = resolve_expr(&self.expr, self.sub);
    }

    pub fn to_int(&self) -> Option<i64> {
        let ValueKind::Int(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_float(&self) -> Option<f64> {
        let ValueKind::Float(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_bool(&self) -> Option<bool> {
        let ValueKind::Bool(b) = self.kind else { return None };
        Some(b)
    }

    pub fn to_char(&self) -> Option<char> {
        let ValueKind::Char(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_hex(&self) -> Option<Hex> {
        let ValueKind::Hex(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_str(&self) -> Option<&str> {
        let ValueKind::Str(i) = &self.kind else { return None };
        Some(i)
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        eprintln!("unsubscribe the value");
    }
}

/// This value can never be part of an evaluation chain, only the return value.
/// It should only ever be the final type that is held by a `Value`, at 
/// the end of an evaluation
#[derive(Debug, PartialEq, PartialOrd)]
pub enum ValueKind<'bp> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Hex(Hex),
    Str(Cow<'bp, str>),
    Map,
    List,
    Composite,
    Null,
}
