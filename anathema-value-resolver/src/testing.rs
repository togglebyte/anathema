use anathema_state::{AnyState, Hex, List, Map, StateId, States, Subscriber};
use anathema_templates::{Expression, Globals, Variables};
use collection::{Collection, CollectionResolver};

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
        F: FnOnce(TestCase<'_>),
    {
        let globals = self.variables.into();
        let state = Box::new(self.state);
        let state_id = self.states.insert(anathema_state::Value::new(state));

        let mut scope = Scope::new();
        scope.insert_state(state_id);
        let case = TestCase {
            globals: &globals,
            scope,
            states: &mut self.states,
            state_id,
        };

        f(case);
    }
}

pub(crate) struct TestCase<'bp> {
    globals: &'bp Globals,
    scope: Scope,
    states: &'bp mut States,
    state_id: StateId,
}

impl<'bp> TestCase<'bp> {
    pub(crate) fn eval(&self, expr: &'bp Expression) -> Value<'bp> {
        let ctx = ResolverCtx::new(&self.globals, &self.scope, self.states);
        let mut resolver = ImmediateResolver::new(&ctx);
        let value_expr = resolver.resolve(expr);
        Value::new(value_expr, Subscriber::ZERO)
    }

    pub(crate) fn eval_collection(&self, expr: &'bp Expression) -> Collection<'bp> {
        let ctx = ResolverCtx::new(&self.globals, &self.scope, self.states);
        let mut resolver = CollectionResolver::new(&ctx);
        let collection_expr = resolver.resolve(expr);
        Collection::new(collection_expr, Subscriber::ZERO)
    }

    pub(crate) fn set_state(&mut self, key: &str, value: impl AnyState) {
        self.with_state(|state| state.insert(key, Box::new(value)));
    }

    pub(crate) fn with_state<F>(&mut self, mut f: F)
        where F: FnOnce(&mut Map<Box<dyn AnyState>>)
    {
        let mut state = self.states.get_mut(self.state_id).unwrap();
        let mut state = state.to_mut_cast::<Map<Box<dyn AnyState>>>();
        f(&mut *state);
    }
}

pub(crate) fn setup() -> TestCaseBuilder {
    let mut states = States::new();
    TestCaseBuilder::new(states)
}
