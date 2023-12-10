use std::ops::Deref;

use crate::map::Map;
use crate::state::State;
use crate::{
    Collection, Context, List, LocalScope, NodeId, Owned, Path, Resolver, StateValue, ValueExpr,
    ValueRef, ValueResolver,
};

#[derive(Debug, crate::State)]
pub struct Inner {
    name: StateValue<String>,
    names: List<String>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            name: StateValue::new("Fiddle McStick".into()),
            names: List::new(vec!["arthur".to_string(), "bobby".into()]),
        }
    }
}

#[derive(Debug, crate::State)]
pub struct TestState {
    pub name: StateValue<String>,
    pub counter: StateValue<usize>,
    pub inner: Inner,
    pub generic_map: Map<Map<usize>>,
    pub generic_list: List<usize>,
    pub nested_list: List<List<usize>>,
    pub debug: StateValue<bool>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            name: StateValue::new("Dirk Gently".to_string()),
            counter: StateValue::new(3),
            inner: Inner::new(),
            generic_map: Map::new([("inner", Map::new([("first", 1), ("second", 2)]))]),
            generic_list: List::new(vec![1, 2, 3]),
            nested_list: List::new(vec![List::new(vec![1, 2, 3])]),
            debug: StateValue::new(false),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Extend value expression -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct TestExpression<T> {
    pub state: Map<T>,
    scope: LocalScope<'static>,
    pub expr: Box<ValueExpr>,
}

impl<T: std::fmt::Debug> TestExpression<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    pub fn eval<'a>(&'a self) -> ValueRef<'a> {
        let context = Context::new(&self.state, &self.scope);
        let mut resolver = Resolver::new(&context, None);
        resolver.resolve(&self.expr)
    }

    pub fn eval_string(&self) -> Option<String> {
        let context = Context::new(&self.state, &self.scope);
        let mut resolver = Resolver::new(&context, None);
        resolver.resolve_string(&self.expr)
    }

    pub fn eval_bool(&self, b: bool) -> bool {
        let context = Context::new(&self.state, &self.scope);
        let mut resolver = Resolver::new(&context, None);
        resolver.resolve_bool(&self.expr) == b
    }

    pub fn expect_owned(self, expected: impl Into<Owned>) {
        let ValueRef::Owned(owned) = self.eval() else {
            panic!("not an owned value")
        };
        assert_eq!(owned, expected.into())
    }
}

impl ValueExpr {
    pub fn with_data<T, K: Into<String>>(
        self,
        inner: impl IntoIterator<Item = (K, T)>,
    ) -> TestExpression<T> {
        let inner = inner.into_iter().map(|(k, v)| (k, v.into()));
        TestExpression {
            state: Map::new(inner),
            expr: Box::new(self),
            scope: LocalScope::empty()
        }
    }

    pub fn test(self) -> TestExpression<usize> {
        TestExpression {
            state: Map::empty(),
            expr: Box::new(self),
            scope: LocalScope::empty()
        }
    }
}

// -----------------------------------------------------------------------------
//   - Paths -
// -----------------------------------------------------------------------------
pub fn ident(p: &str) -> Box<ValueExpr> {
    ValueExpr::Ident(p.into()).into()
}

pub fn index(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Index(lhs, rhs).into()
}

pub fn dot(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Dot(lhs, rhs).into()
}

// -----------------------------------------------------------------------------
//   - Maths -
// -----------------------------------------------------------------------------
pub fn mul(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Mul(lhs, rhs).into()
}

pub fn div(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Div(lhs, rhs).into()
}

pub fn modulo(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Mod(lhs, rhs).into()
}

pub fn sub(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Sub(lhs, rhs).into()
}

pub fn add(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Add(lhs, rhs).into()
}

// -----------------------------------------------------------------------------
//   - Values -
// -----------------------------------------------------------------------------
pub fn unum(int: u64) -> Box<ValueExpr> {
    ValueExpr::Owned(Owned::from(int)).into()
}

pub fn inum(int: i64) -> Box<ValueExpr> {
    ValueExpr::Owned(Owned::from(int)).into()
}

pub fn boolean(b: bool) -> Box<ValueExpr> {
    ValueExpr::Owned(Owned::from(b)).into()
}

pub fn strlit(lit: &str) -> Box<ValueExpr> {
    ValueExpr::String(lit.into()).into()
}

// -----------------------------------------------------------------------------
//   - List -
// -----------------------------------------------------------------------------
pub fn list<E: Into<ValueExpr>>(input: impl IntoIterator<Item = E>) -> Box<ValueExpr> {
    let vec = input.into_iter().map(|val| val.into()).collect::<Vec<_>>();
    ValueExpr::List(vec.into()).into()
}

// -----------------------------------------------------------------------------
//   - Op -
// -----------------------------------------------------------------------------
pub fn neg(expr: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Negative(expr).into()
}

pub fn not(expr: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Not(expr).into()
}

pub fn eq(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Equality(lhs, rhs).into()
}

pub fn and(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::And(lhs, rhs).into()
}

pub fn or(lhs: Box<ValueExpr>, rhs: Box<ValueExpr>) -> Box<ValueExpr> {
    ValueExpr::Or(lhs, rhs).into()
}
