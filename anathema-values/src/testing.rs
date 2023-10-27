
use crate::map::Map;
use crate::{
    Context, List, NodeId, Owned, Path, Scope, State, StateValue, 
    ValueExpr, ValueRef, Collection
};

#[derive(Debug)]
pub struct Inner {
    name: StateValue<String>,
    names: List<String>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            name: StateValue::new("Fiddle McStick".into()),
            names: List::empty(),
        }
    }
}

impl State for Inner {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match key {
            Path::Key(s) => match s.as_str() {
                "name" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.name.subscribe(node_id);
                    }
                    Some((&self.name).into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<usize> {
        todo!()
    }
}

#[derive(Debug)]
pub struct TestState {
    pub name: StateValue<String>,
    pub counter: StateValue<usize>,
    pub inner: Inner,
    pub generic_map: StateValue<Map<Map<usize>>>,
    pub generic_list: StateValue<List<List<usize>>>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            name: StateValue::new("Dirk Gently".to_string()),
            counter: StateValue::new(0),
            inner: Inner::new(),
            generic_map: StateValue::new(Map::new([(
                "inner",
                Map::new([("first", 1), ("second", 2)]),
            )])),
            generic_list: StateValue::new(List::new(vec![
                List::new(vec![1, 2, 3]),
            ])),
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
                "generic_map" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.generic_map.subscribe(node_id);
                    }
                    let map = ValueRef::Map(&self.generic_map.inner);
                    Some(map)
                }
                _ => None,
            },
            Path::Composite(lhs, rhs) => match &**lhs {
                Path::Key(key) if key == "inner" => self.inner.get(rhs, node_id),
                Path::Key(key) if key == "generic_map" => self.generic_map.get(rhs, node_id),
                Path::Key(key) if key == "generic_list" => self.generic_list.get(rhs, node_id),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<usize> {
        match key {
            Path::Key(s) => match s.as_str() {
                "generic_list" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.generic_list.subscribe(node_id);
                    }
                    Some(self.generic_list.len())
                }
                _ => None
            }
            _ => None
        }
    }
}

// -----------------------------------------------------------------------------
//   - Extend scope functionality for testing -
// -----------------------------------------------------------------------------
impl<const N: usize> From<[(&'static str, Owned); N]> for Scope<'_> {
    fn from(values: [(&'static str, Owned); N]) -> Self {
        let mut scope = Self::new(None);
        for (key, value) in values {
            scope.scope(
                key.into(),
                ValueRef::Owned(value.into()),
            );
        }

        scope
    }
}

// -----------------------------------------------------------------------------
//   - Extend value expression -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct TestExpression<'a, S> {
    pub state: S,
    pub scope: Scope<'a>,
    expr: Box<ValueExpr>,
}

impl<'a, S: State> TestExpression<'a, S> {
    pub fn eval(&'a self) -> Option<ValueRef<'a>> {
        let context = Context::new(&self.state, &self.scope);
        let node_id = 0.into();
        self.expr.eval_value(&context, Some(&node_id))
    }

    pub fn eval_string(&'a self) -> Option<String> {
        let context = Context::new(&self.state, &self.scope);
        let node_id = 0.into();
        self.expr.eval_string(&context, Some(&node_id))
    }

    pub fn expect_owned(self, expected: impl Into<Owned>) {
        let ValueRef::Owned(owned) = self.eval().unwrap() else {
            panic!("not an owned value")
        };
        assert_eq!(owned, expected.into())
    }
}

impl ValueExpr {
    #[cfg(feature = "testing")]
    pub fn test<'a>(self, scope: impl Into<Scope<'a>>) -> TestExpression<'a, TestState> {
        let scope = scope.into();

        TestExpression {
            scope,
            state: TestState::new(),
            expr: self.into(),
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
