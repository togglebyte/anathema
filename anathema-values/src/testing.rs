use std::borrow::Cow;

use crate::{List, NodeId, Owned, Path, State, StateValue, Value, ValueExpr, ValueRef};

#[derive(Debug)]
struct Inner {
    name: StateValue<String>,
    names: List<String>,
}

#[derive(Debug)]
pub struct TestState {
    name: StateValue<String>,
    counter: StateValue<usize>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            name: StateValue::new("Dirk Gently".to_string()),
            counter: StateValue::new(0),
        }
    }
}

impl State for TestState {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match key {
            Path::Key(s) => match s.as_str() {
                "name" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.name.subscribe(node_id);
                    }
                    Some((&self.name).into())
                }
                "counter" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.name.subscribe(node_id);
                    }
                    Some((&self.counter).into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<crate::Collection> {
        todo!()
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
    ValueExpr::Sub(lhs, rhs).into()
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
    ValueExpr::Value(Value::Owned(Owned::from(int))).into()
}

pub fn inum(int: i64) -> Box<ValueExpr> {
    ValueExpr::Value(Value::Owned(Owned::from(int))).into()
}

pub fn bool(b: bool) -> Box<ValueExpr> {
    ValueExpr::Value(Value::Owned(Owned::from(b))).into()
}

// -----------------------------------------------------------------------------
//   - List -
// -----------------------------------------------------------------------------
pub fn list<E: Into<ValueExpr>>(input: impl IntoIterator<Item = E>) -> Box<ValueExpr> {
    let vec = input.into_iter().map(|val| val.into()).collect::<Vec<_>>();
    ValueExpr::List(vec.into()).into()
}

