use std::marker::PhantomData;

use anathema_values::{BucketRef, Listen, ScopeId};

use crate::{NodeId, FromContext};

pub struct DataCtx<'a, T: FromContext> {
    bucket: &'a BucketRef<'a, T::Value>,
    node_id: &'a NodeId,
    scope: Option<ScopeId>,
    inner: &'a T::Ctx,
    _p: PhantomData<T::Notifier>,
}

impl<'a, T: FromContext> DataCtx<'a, T> {
    pub fn new(
        bucket: &'a BucketRef<'a, T::Value>,
        node_id: &'a NodeId,
        scope: Option<ScopeId>,
        inner: &'a T::Ctx,
    ) -> Self {
        Self {
            bucket,
            node_id,
            scope,
            inner,
            _p: PhantomData,
        }
    }

    pub fn get(&self, key: &str) -> Option<()> {
        let path = self.bucket.get_path(key)?;
        let val = self.bucket.by_path_or_empty(path, self.scope);
        // subscribe to value
        T::Notifier::subscribe(val, self.node_id.clone());
        Some(())
    }
}
