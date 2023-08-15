use std::marker::PhantomData;
use std::sync::Arc;

use anathema_values::hashmap::HashMap;
use anathema_values::{StoreRef, Container, Listen, PathId, ReadOnly, ScopeId, ValueRef, ScopeValue};

use crate::NodeId;

// -----------------------------------------------------------------------------
//   - Expression attributes -
// -----------------------------------------------------------------------------
pub struct ExpressionValues<T> {
    inner: HashMap<String, ExpressionValue<T>>,
}

impl<T: Clone> ExpressionValues<T> {
    pub fn empty() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: ExpressionValue<T>) {
        self.inner.insert(key.into(), value);
    }

    pub(crate) fn get(&self, key: impl AsRef<str>) -> Option<&ExpressionValue<T>> {
        self.inner.get(key.as_ref())
    }
}

// -----------------------------------------------------------------------------
//   - Expression attribute -
// -----------------------------------------------------------------------------
pub enum ExpressionValue<T> {
    Dyn(PathId),
    Static(Arc<T>),
    List(Box<[ExpressionValue<T>]>),
}

impl<T> ExpressionValue<T> {
    pub fn stat(val: T) -> Self {
        Self::Static(Arc::new(val))
    }

    pub(crate) fn to_scope_value<N: Listen<Value = T, Key = NodeId>>(
        &self,
        store: &StoreRef<'_, T>,
        scope: Option<ScopeId>,
        node_id: &NodeId,
    ) -> ScopeValue<T> {
        match self {
            Self::Dyn(path_id) => {
                let val = store.by_path_or_empty(*path_id, scope);
                if let ScopeValue::Dyn(val) = val {
                    // NOTE: this is most certainly going to lead to bugs
                    // Here is where we subscribe node kinds like for loops and controlflow.
                    // There are really only two places where we subscribe to values: here, and
                    // when we load attributes for the widgets.
                    N::subscribe(val, node_id.clone());
                }
                val
            }
            Self::Static(val) => ScopeValue::Static(val.clone()),
            Self::List(values) => ScopeValue::List(
                values
                    .iter()
                    .map(|val| val.to_scope_value::<N>(store, scope, node_id))
                    .collect::<Box<_>>(),
            ),
        }
    }
}

impl<T> Clone for ExpressionValue<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Dyn(path) => Self::Dyn(*path),
            Self::Static(arc) => Self::Static(arc.clone()),
            Self::List(list) => Self::List(list.clone()),
        }
    }
}
