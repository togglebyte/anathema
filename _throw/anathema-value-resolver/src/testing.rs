use anathema_state::{AnyMap, List, Map, Maybe, State, StateId, States, Subscriber, Value};
use anathema_store::slab::Key;
use anathema_templates::Variables;
use anathema_templates::expressions::{ExpressionId, Expressions};

use super::*;
use crate::context::ResolverCtx;
use crate::scope::Scope;

#[derive(Debug)]
pub(crate) struct TestState {
    pub(crate) string: Value<&'static str>,
    pub(crate) num: Value<i32>,
    pub(crate) num_2: Value<i32>,
    pub(crate) float: Value<f64>,
    pub(crate) list: Value<List<&'static str>>,
    pub(crate) map: Value<Map<i32>>,
    pub(crate) opt_map: Value<Maybe<Map<i32>>>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            string: "".into(),
            num: 0.into(),
            num_2: 0.into(),
            float: 0.0.into(),
            list: List::empty().into(),
            map: Map::empty().into(),
            opt_map: Value::new(Maybe::none()),
        }
    }
}

impl State for TestState {
    fn type_info(&self) -> anathema_state::Type {
        anathema_state::Type::Composite
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap> {
        Some(self)
    }
}

impl AnyMap for TestState {
    fn lookup(&self, key: &str) -> Option<anathema_state::PendingValue> {
        match key {
            "list" => self.list.reference().into(),
            "num" => self.num.reference().into(),
            "num_2" => self.num_2.reference().into(),
            "float" => self.float.reference().into(),
            "string" => self.string.reference().into(),
            "map" => self.map.reference().into(),
            "opt_map" => self.opt_map.reference().into(),
            _ => None,
        }
    }

    fn is_empty(&self) -> bool {
        false
    }
}

pub(crate) struct TestCase<'a, 'bp> {
    variables: &'static mut Variables,
    states: &'a mut States,
    pub attributes: AttributeStorage<'bp>,
    function_table: &'static FunctionTable,
}

impl<'a, 'bp> TestCase<'a, 'bp> {
    pub fn new(states: &'a mut States, variables: Variables) -> Self {
        let mut attributes = AttributeStorage::empty();
        attributes.insert(Key::ZERO, Attributes::empty());

        Self {
            variables: Box::leak(variables.into()),
            states,
            attributes,
            function_table: Box::leak(FunctionTable::new().into()),
        }
    }

    pub(crate) fn eval(&mut self, expressions: &'bp Expressions, expr: ExpressionId) -> crate::value::Value<'bp> {
        let state_id = StateId::ZERO;
        let scope = Scope::with_component(state_id, Key::ZERO, None);

        let mut ctx = ResolverCtx::new(
            self.variables,
            &scope,
            self.states,
            &self.attributes,
            self.function_table,
            expressions,
        );
        resolve(expr, &mut ctx, Subscriber::ZERO)
    }

    pub fn set_attribute(&mut self, key: &'bp str, value: impl Into<ValueKind<'bp>>) {
        self.attributes
            .with_mut(Key::ZERO, |attributes, _| attributes.set(key, value));
    }

    pub(crate) fn with_state<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut TestState) -> U,
    {
        let state = self.states.get_mut(StateId::ZERO).unwrap();
        let mut state = state.to_mut_cast::<TestState>();
        f(&mut state)
    }

    pub(crate) fn register_global(&mut self, key: &str, id: ExpressionId) {
        self.variables.define_global(key, id).unwrap();
    }
}

pub(crate) fn setup<'bp, F>(states: &mut States, variables: Variables, mut f: F)
where
    F: FnMut(&mut TestCase<'_, 'bp>),
{
    states.insert(Box::new(TestState::new()));
    let mut test = TestCase::new(states, variables);
    f(&mut test)
}
