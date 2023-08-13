use std::marker::PhantomData;
use std::ops::Deref;

use anathema_values::{StoreRef, Listen, ScopeId};

use crate::attribute::ExpressionValue;
use crate::{FromContext, NodeId, Attribute, ExpressionValue};

pub struct DataCtx<'a, T: FromContext> {
    pub bucket: &'a StoreRef<'a, T::Value>,
    node_id: &'a NodeId,
    scope: Option<ScopeId>,
    inner: &'a T::Ctx,
    attributes: &'a ExpressionValue<T::Value>,
    _p: PhantomData<T::Notifier>,
}

impl<'a, T: FromContext> DataCtx<'a, T> {
    pub fn new(
        bucket: &'a StoreRef<'a, T::Value>,
        node_id: &'a NodeId,
        scope: Option<ScopeId>,
        inner: &'a T::Ctx,
        attributes: &'a ExpressionValue<T::Value>,
    ) -> Self {
        Self {
            bucket,
            node_id,
            scope,
            inner,
            attributes,
            _p: PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> Attribute<T::Value> {
        match self.attributes.get(key) {
            Some(ExpressionValue::Dyn(path)) => {
                let val = self.bucket.by_path_or_empty(*path, self.scope);
                // subscribe to value
                T::Notifier::subscribe(val, self.node_id.clone());
                Attribute::Dyn(val)
            }
            None => {
                let path = self.bucket.get_or_insert_path(key);
                let val = self.bucket.by_path_or_empty(path, self.scope);
                // subscribe to value
                T::Notifier::subscribe(val, self.node_id.clone());
                Attribute::Dyn(val)
            }
            Some(ExpressionValue::Static(value)) => Attribute::Static(value.clone()),
        }
    }
}

impl<'a, T: FromContext> Deref for DataCtx<'a, T> {
    type Target = T::Ctx;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
