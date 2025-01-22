use anathema_state::{AnyState, Hex, List, Map, StateId, States, SubTo, Subscriber};
use anathema_store::slab::Key;
use anathema_templates::{Expression, Globals, Variables};
use expression::ValueExpr;

use super::*;
use crate::context::ResolverCtx;
use crate::immediate::Resolver;
use crate::scope::Scope;
use crate::value::Value;

pub(crate) struct TestCase<'a, 'bp> {
    globals: &'static Globals,
    states: &'a mut States,
    pub attributes: AttributeStorage<'bp>,
}

impl<'a, 'bp> TestCase<'a, 'bp> {
    pub fn new(states: &'a mut States, globals: Globals) -> Self {
        let mut attributes = AttributeStorage::empty();
        attributes.insert(Key::ZERO, Attributes::empty(Key::ZERO));

        Self {
            globals: Box::leak(globals.into()),
            states,
            attributes,
        }
    }

    pub(crate) fn eval(&self, expr: &'bp Expression) -> Value<'bp> {
        let root = Scope::root();
        let state_id = StateId::ZERO;
        let mut scope = Scope::with_component(state_id, Key::ZERO, &root);

        let ctx = ResolverCtx::new(&self.globals, &scope, &self.states, &self.attributes);
        let mut resolver = Resolver::new(&ctx);
        let value_expr = resolver.resolve(expr);
        let value = Value::new(value_expr, Subscriber::ZERO, ctx.attribute_storage);
        value
    }

    pub fn set_attribute(&mut self, key: &'bp str, value: Value<'bp>) {
        let attributes = self.attributes.get_mut(Key::ZERO);
        attributes.set(key, value);
    }

    pub(crate) fn with_state<F>(&mut self, mut f: F)
    where
        F: FnOnce(&mut Map<Box<dyn AnyState>>),
    {
        let mut state = self.states.get_mut(StateId::ZERO).unwrap();
        let mut state = state.to_mut_cast::<Map<Box<dyn AnyState>>>();
        f(&mut *state);
    }

    pub(crate) fn set_state(&mut self, key: &str, value: impl AnyState) {
        self.with_state(|state| state.insert(key, Box::new(value)));
    }
}

pub(crate) fn setup<'bp, F>(states: &mut States, globals: Variables, mut f: F)
where
    F: FnMut(&mut TestCase<'_, 'bp>),
{
    let state: Map<Box<dyn AnyState>> = Map::empty();
    let state = Box::new(state);
    states.insert(anathema_state::Value::new(state));
    let mut test = TestCase::new(states, globals.into());
    f(&mut test)
}
