use std::marker::PhantomData;

use anathema_geometry::Size;
use anathema_state::{Map, State, StateId, States};
use anathema_store::tree::TreeForEach;
use anathema_templates::{Expression, Globals};

use crate::expressions::{eval, EvalValue};
use crate::layout::{Constraints, LayoutCtx, LayoutFilter, PositionCtx};
use crate::scope::{Scope, ScopeLookup};
use crate::values::{ValueId, ValueIndex};
use crate::{AttributeStorage, Factory, PositionChildren, Value, Widget, WidgetId, WidgetKind};

pub struct NoExpr;
pub struct WithExpr(Expression);

pub struct ScopedTest<T, S> {
    _p: PhantomData<T>,
    test_state: S,
    states: States,
}

impl<T: 'static + State> ScopedTest<T, NoExpr> {
    pub fn new() -> Self {
        let mut states = States::new();
        let map = Map::<T>::empty();
        states.insert(Box::new(map));
        Self {
            _p: PhantomData,
            test_state: NoExpr,
            states,
        }
    }

    pub fn with_expr(self, expr: impl Into<Expression>) -> ScopedTest<T, WithExpr> {
        ScopedTest {
            _p: PhantomData,
            test_state: WithExpr(expr.into()),
            states: self.states,
        }
    }
}

impl<T: 'static + State, S> ScopedTest<T, S> {
    pub fn with_value(mut self, key: &str, value: T) -> Self {
        let map = self.states.get_mut(StateId::ZERO).unwrap();
        let map = map
            .to_any_mut()
            .downcast_mut::<anathema_state::Value<Map<T>>>()
            .unwrap();
        map.insert(key, value);
        self
    }

    pub fn lookup<F>(self, lookup: ScopeLookup<'_>, f: F)
    where
        F: FnOnce(EvalValue<'_>),
    {
        let mut scope = Scope::new();
        scope.insert_state(StateId::ZERO);
        let value = scope
            .get(lookup, &mut None, &self.states)
            .expect("should contain value");
        f(value);
    }
}

impl<T: 'static + State> ScopedTest<T, WithExpr> {
    pub fn eval<F>(&mut self, f: F)
    where
        F: FnOnce(Value<'_, EvalValue<'_>>),
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let key = WidgetId::from((NEXT_ID.fetch_add(1, Ordering::Relaxed), 0));
        let index = ValueIndex::ZERO;
        let value_id = ValueId::from((key, index));
        let mut scope = Scope::new();
        let globals = Globals::new(Default::default());
        scope.insert_state(StateId::ZERO);
        let value = eval(&self.test_state.0, &globals, &scope, &self.states, value_id);
        f(value)
    }
}

#[derive(Debug, Default)]
struct TestWidget;

impl Widget for TestWidget {
    fn layout(
        &mut self,
        _children: TreeForEach<'_, '_, WidgetKind<'_>, LayoutFilter<'_, '_>>,
        _: Constraints,
        _: WidgetId,
        _: &mut LayoutCtx<'_, '_>,
    ) -> Size {
        todo!()
    }

    fn position<'bp>(
        &mut self,
        _children: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _ctx: PositionCtx,
    ) {
        todo!()
    }
}

pub(crate) fn setup_test_factory() -> Factory {
    let mut fac = Factory::new();
    fac.register_default::<TestWidget>("test");
    fac
}
