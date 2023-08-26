use std::marker::PhantomData;
use std::ops::Deref;

use anathema_values::{Listen, ScopeId, ScopeValue, StoreRef, ValueRef, Container};

use crate::{ExpressionValue, ExpressionValues, FromContext, NodeId};

pub struct DataCtx<'a, T: FromContext> {
    pub store: &'a StoreRef<'a, T::Value>,
    pub node_id: &'a NodeId,
    pub scope: Option<ScopeId>,
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
            store: bucket,
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
    /// This will subscribe the widget node to a value as long as the path exists.
    pub fn get(&self, key: &str) -> Option<ScopeValue<T::Value>> {
        let val = self.attributes.get(key)?;
        let val = val.to_scope_value::<T::Notifier>(self.store, self.scope, self.node_id);
        Some(val)
    }

    pub fn by_ref(&self, value_ref: ValueRef<Container<T::Value>>) -> Option<Container<T::Value>> {
        // TODO: Second place we can subscribe to changes.
        //       This is a nightmare.
        T::Notifier::subscribe(value_ref, self.node_id.clone());
        self.store.read().get(value_ref).cloned()
    }
}

impl<'a, T: FromContext> Deref for DataCtx<'a, T> {
    type Target = T::Ctx;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
