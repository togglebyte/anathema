use anathema_state::{AnyState, Hex, List, Map, StateId, States, Subscriber};
use anathema_store::slab::Key;
use anathema_templates::{Expression, Globals, Variables};

use super::*;
use crate::context::ResolverCtx;
use crate::immediate::ImmediateResolver;
use crate::scope::Scope;
use crate::value::Value;
use crate::Resolver;

pub(crate) struct TestCaseBuilder {
    variables: Variables,
    states: States,
    state: Map<Box<dyn AnyState>>,
}

impl TestCaseBuilder {
    pub fn new(states: States) -> Self {
        Self {
            variables: Variables::new(),
            states,
            state: Map::empty(),
        }
    }

    pub fn with_global(mut self, key: &str, value: impl Into<Expression>) -> Self {
        self.variables.declare(key, value);
        self
    }

    pub fn finish<F>(mut self, mut f: F)
    where
        F: FnOnce(TestCase<'_, '_>),
    {
        let globals = self.variables.into();
        let state = Box::new(self.state);
        let state_id = self.states.insert(anathema_state::Value::new(state));
        let mut attributes = AttributeStorage::empty();

        let attributes = AttributeStorage::empty();
        let root = Scope::root();
        let mut scope = Scope::with_component(state_id, Key::ZERO, &root);
        let case = TestCase {
            globals: &globals,
            attributes: &attributes,
            scope: &scope,
            states: &mut self.states,
            state_id,
        };

        f(case);
    }
}

pub(crate) struct TestCase<'frame, 'bp> {
    globals: &'bp Globals,
    attributes: &'bp AttributeStorage<'bp>,
    scope: &'frame Scope<'frame, 'bp>,
    states: &'bp mut States,
    state_id: StateId,
}

impl<'bp> TestCase<'_, 'bp> {
    pub(crate) fn eval(&self, expr: &'bp Expression) -> Value<'bp> {
        let ctx = ResolverCtx::new(&self.globals, &self.scope, self.states, self.attributes);
        let mut resolver = ImmediateResolver::new(&ctx);
        let value_expr = resolver.resolve(expr);
        Value::new(value_expr, Subscriber::ZERO)
    }

    pub(crate) fn eval_collection(&self, expr: &'bp Expression) -> Collection<'bp> {
        let value = self.eval(expr);
        Collection(value)
    }

    pub(crate) fn set_state(&mut self, key: &str, value: impl AnyState) {
        self.with_state(|state| state.insert(key, Box::new(value)));
    }

    pub(crate) fn with_state<F>(&mut self, mut f: F)
    where
        F: FnOnce(&mut Map<Box<dyn AnyState>>),
    {
        let mut state = self.states.get_mut(self.state_id).unwrap();
        let mut state = state.to_mut_cast::<Map<Box<dyn AnyState>>>();
        f(&mut *state);
    }

    pub(crate) fn set_attribute(&mut self, key: &str, value: ()) {
    }
}

pub(crate) fn setup() -> TestCaseBuilder {
    let mut states = States::new();
    TestCaseBuilder::new(states)
}
