use std::borrow::Cow;

use anathema_state::Subscriber;

use crate::expression::{resolve_expr, ValueExpr};

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

    pub fn to_int(self) -> Option<i64> {
        let ValueKind::Int(i) = self.kind else { return None };
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
#[derive(Debug)]
pub enum ValueKind<'bp> {
    Str,
    Int(i64),
    Float,
    Bool,
    Char,
    String(Cow<'bp, str>),
    Map,
    List,
    Composite,
    Null,
}
