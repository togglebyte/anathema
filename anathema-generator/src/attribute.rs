use std::marker::PhantomData;
use std::sync::Arc;

use anathema_values::hashmap::HashMap;
use anathema_values::{StoreRef, Container, Listen, PathId, ReadOnly, ScopeId, ValueRef};

use crate::NodeId;

// -----------------------------------------------------------------------------
//   - Expression attributes -
// -----------------------------------------------------------------------------
pub struct ExpressionAttributes<T> {
    inner: HashMap<String, ExpressionValue<T>>,
}

impl<T> ExpressionAttributes<T> {
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
    pub fn single(val: T) -> Self {
        Self::Static(Arc::new(Container::Single(val)))
    }

    pub(crate) fn to_attrib<N: Listen<Value = T, Key = NodeId>>(
        &self,
        bucket: &StoreRef<'_, T>,
        scope: Option<ScopeId>,
        node_id: &NodeId,
    ) -> Attribute<T> {
        match self {
            Self::Dyn(path_id) => {
                let val = bucket.by_path_or_empty(*path_id, scope);
                // TODO: this is 100% going to lead to a hard-to-find bug
                N::subscribe(val, node_id.clone());
                Attribute::Dyn(val)
            }
            Self::Static(val) => Attribute::Static(val.clone()),
            Self::List(values) => Attribute::List(
                values
                    .iter()
                    .map(|val| val.to_attrib::<N>(bucket, scope, node_id))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl<T> Clone for ExpressionValue<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Dyn(path_id) => Self::Dyn(*path_id),
            Self::Static(val) => Self::Static(Arc::clone(val)),
            Self::List(val) => Self::List(Arc::clone(val)),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Attribute -
// -----------------------------------------------------------------------------
pub enum Attribute<T> {
    Dyn(ValueRef<T>),
    Static(Arc<T>),
    List(Box<[Attribute<T>]>),
}

impl<T> Attribute<T> {
    // pub fn load<'a>(&'a self, bucket: &'a ReadOnly<'a, T>) -> Option<&Container<T>> {
    //     match self {
    //         Self::Static(val) => Some(val),
    //         Self::Dyn(val) => bucket.get(*val),
    //         Self::List(list) => Some(list)
    //     }
    // }
}

impl<T> From<ValueRef<Container<T>>> for Attribute<T> {
    fn from(value: ValueRef<Container<T>>) -> Self {
        Self::Dyn(value)
    }
}

impl<T> From<Arc<Container<T>>> for Attribute<T> {
    fn from(value: Arc<Container<T>>) -> Self {
        Self::Static(Arc::clone(&value))
    }
}
