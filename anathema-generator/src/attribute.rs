use std::marker::PhantomData;
use std::sync::Arc;

use anathema_values::hashmap::HashMap;
use anathema_values::{BucketRef, Container, Listen, PathId, ScopeId, ValueRef, ReadOnly};

use crate::NodeId;

// -----------------------------------------------------------------------------
//   - Expression attributes -
// -----------------------------------------------------------------------------
pub struct ExpressionAttributes<T> {
    inner: HashMap<String, ExpressionAttribute<T>>,
}

impl<T> ExpressionAttributes<T> {
    pub fn empty() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: ExpressionAttribute<T>) {
        self.inner.insert(key.into(), value);
    }

    pub(crate) fn get(&self, key: impl AsRef<str>) -> Option<&ExpressionAttribute<T>> {
        self.inner.get(key.as_ref())
    }
}

// -----------------------------------------------------------------------------
//   - Expression attribute -
// -----------------------------------------------------------------------------
pub enum ExpressionAttribute<T> {
    Dyn(PathId),
    Static(Arc<Container<T>>),
}

impl<T> ExpressionAttribute<T> {
    pub fn single(val: T) -> Self {
        Self::Static(Arc::new(Container::Single(val)))
    }

    pub(crate) fn to_attrib<N: Listen<Value = T, Key = NodeId>>(
        &self,
        bucket: &BucketRef<'_, T>,
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
        }
    }
}

impl<T> Clone for ExpressionAttribute<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Dyn(path_id) => Self::Dyn(*path_id),
            Self::Static(val) => Self::Static(Arc::clone(val)),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Attribute -
// -----------------------------------------------------------------------------
pub enum Attribute<T> {
    Dyn(ValueRef<Container<T>>),
    Static(Arc<Container<T>>),
}

impl<T> Attribute<T> {
    pub fn load<'a>(&'a self, bucket: &'a ReadOnly<'a, T>) -> Option<&T> {
        match self {
            Self::Static(val) => {
                let val: &Container<_> = &*val;
                match val {
                    Container::Single(val) => Some(val),
                    _ => None
                }
            }
            Self::Dyn(val) => {
                let val = &*bucket.get(*val)?;
                match val {
                    Container::Single(val) => Some(val),
                    _ => None
                }
            },
        }
    }
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
