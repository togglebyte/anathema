use std::marker::PhantomData;

use anathema_values::{BucketRef, Listen, ScopeId};

use crate::NodeId;

pub struct DataCtx<'a, Val, InnerCtx, L> {
    bucket: &'a BucketRef<'a, Val>,
    node_id: &'a NodeId,
    scope: Option<ScopeId>,
    inner: InnerCtx,
    _p: PhantomData<L>,
}

impl<'a, Val, InnerCtx, L> DataCtx<'a, Val, InnerCtx, L>
where
    L: Listen<Value = Val, Key = NodeId>,
{
    pub fn new(
        bucket: &'a BucketRef<'a, Val>,
        node_id: &'a NodeId,
        scope: Option<ScopeId>,
        inner: InnerCtx,
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
        L::subscribe(val, self.node_id.clone());
        Some(())
    }
}
