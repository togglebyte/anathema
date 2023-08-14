use std::marker::PhantomData;
use std::ops::Deref;

use anathema_values::{Listen, ScopeId, ScopeValue, StoreRef};

use crate::{ExpressionValue, ExpressionValues, FromContext, NodeId};

pub struct DataCtx<'a, T: FromContext> {
    pub bucket: &'a StoreRef<'a, T::Value>,
    node_id: &'a NodeId,
    scope: Option<ScopeId>,
    inner: &'a T::Ctx,
    attributes: &'a ExpressionValues<T::Value>,
    _p: PhantomData<T::Notifier>,
}

impl<'a, T: FromContext> DataCtx<'a, T> {
    pub fn new(
        bucket: &'a StoreRef<'a, T::Value>,
        node_id: &'a NodeId,
        scope: Option<ScopeId>,
        inner: &'a T::Ctx,
        attributes: &'a ExpressionValues<T::Value>,
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

    /// Get the value for widget attribute.
    /// This is used when composing widgets via the `from_context`.
    ///
    /// This will subscribe the widget node to a value.
    pub fn get(&self, key: &str) -> ScopeValue<T::Value> {
        match self.attributes.get(key) {
            // Some(ExpressionValue::Dyn(path)) => {
            //     let val = self.bucketAttribute::Static(value.clone()),.by_path_or_empty(*path, self.scope);
            //     // subscribe to value
            //     if let ScopeValue::Dyn(val) = val {
            //         T::Notifier::subscribe(val, self.node_id.clone());
            //     }
            //     val
            // }
            Some(val) => val.to_scope_value::<T::Notifier>(self.bucket, self.scope, self.node_id),
            None => {
                let path = self.bucket.get_or_insert_path(key);
                let val = self.bucket.by_path_or_empty(path, self.scope);
                // subscribe to value
                if let ScopeValue::Dyn(val) = val {
                    T::Notifier::subscribe(val, self.node_id.clone());
                }
                val
            }
        }
    }
}

impl<'a, T: FromContext> Deref for DataCtx<'a, T> {
    type Target = T::Ctx;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
