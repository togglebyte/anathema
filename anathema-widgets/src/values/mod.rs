use std::ops::{Deref, DerefMut};

use anathema_state::ValueRef;
use anathema_store::smallmap::{SmallIndex, SmallMap};
use anathema_templates::Expression;

use crate::expressions::{Either, EvalValue};
use crate::widget::ValueKey;
use crate::Scope;

pub(crate) type ValueId = anathema_state::Subscriber;
pub type ValueIndex = SmallIndex;
pub type Values<'bp> = SmallMap<ValueKey<'bp>, Value<'bp, EvalValue<'bp>>>;

/// A value that can be re-evaluated in the future.
///
/// A widget may contain a value that doesn't yet exist but may exist
/// in the future.
/// E.g a value in a `Map`. If the value is added at a later stage
/// then the [`Value`] can be resolved again once the value is present in the map.
///
/// This is used in combination with `anathema_state::register_future`.
/// ```ignore
/// # use anathema_widgets::Scope;
/// # use anathema_templates::Expression;
/// # use anathema_state::Map;
/// let scope = Scope::new();
/// let expr = Expression::Ident("val".into());
///
/// let mut value = eval(&expr, &scope, (0, 0));
///
/// let mut state = Map::empty();
/// state.insert("val", 123);
///
/// value = eval(&expr, &scope, (0, 0));
/// ```
#[derive(Debug)]
pub struct Value<'bp, T> {
    inner: T,
    pub(crate) expr: Option<&'bp Expression>,
}

impl<'bp, T> Value<'bp, T> {
    pub fn new(inner: T, expr: Option<&'bp Expression>) -> Self {
        Self { inner, expr }
    }

    pub(crate) fn inner(&self) -> &T {
        &self.inner
    }
}

impl<'bp> Value<'bp, EvalValue<'bp>> {
    pub(crate) fn load_common_val(&self) -> Option<Either<'_>> {
        self.inner.load_common_val()
    }

    /// Re-evaluate the value if it has been removed.
    /// This will replace the inner value with an empty EvalValue
    /// and register the value for future changes
    pub(crate) fn reload_val(
        &mut self,
        id: ValueId,
        globals: &'bp anathema_templates::Globals,
        scope: &Scope<'bp>,
        states: &anathema_state::States,
    ) {
        if !self.inner.contains_index() {
            return;
        }
        let Some(expr) = self.expr else { return };
        let Value { inner, .. } = crate::expressions::eval(expr, globals, scope, states, id);
        self.inner = inner;
    }
}

impl<'bp, T> Deref for Value<'bp, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'bp, T> DerefMut for Value<'bp, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'bp, T> From<T> for Value<'bp, EvalValue<'bp>>
where
    EvalValue<'bp>: From<T>,
{
    fn from(value: T) -> Self {
        let value: EvalValue<'_> = value.into();
        Self {
            inner: value,
            expr: None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Collection<'bp> {
    /// A static list of expression.
    /// The expressions them selves are not necessarily
    /// static, but the collection it self will not change.
    /// ```text
    /// for x in [1, 2, state_value]
    ///     text x
    /// ```
    Static(Box<[EvalValue<'bp>]>),
    /// This will (probably) resolve to a collection from a state.
    Dyn(ValueRef),
    /// Index value.
    #[allow(dead_code)]
    Index(Box<Collection<'bp>>, Box<EvalValue<'bp>>),
    /// This value doesn't exist now, but might exist in the future.
    /// See [`nodes::future::try_resolve_value`].
    Future,
}

impl<'bp> Collection<'bp> {
    pub(crate) fn count(&self) -> usize {
        match self {
            Self::Static(e) => e.len(),
            Self::Dyn(value_ref) => value_ref.as_state().map(|state| state.count()).unwrap_or(0),
            Self::Index(collection, _) => collection.count(),
            Self::Future => 0,
        }
    }

    pub(crate) fn scope(&self, scope: &mut Scope<'bp>, binding: &'bp str, index: usize) {
        match self {
            Collection::Static(expressions) => {
                let downgrade = expressions[index].downgrade();
                scope.scope_downgrade(binding, downgrade);
            }
            Collection::Dyn(value_ref) => {
                let value = value_ref
                    .as_state()
                    .and_then(|state| state.state_lookup(index.into()))
                    .unwrap(); // TODO: unwrap...
                scope.scope_pending(binding, value)
            }
            Collection::Index(collection, _) => collection.scope(scope, binding, index),
            Collection::Future => {}
        }
    }
}
