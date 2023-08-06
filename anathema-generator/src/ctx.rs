use anathema_values::{BucketRef, Listen, ScopeId};

use crate::NodeId;

pub struct DataCtx<'a, Val, L> {
    bucket: &'a BucketRef<'a, Val>,
    node_id: &'a NodeId,
    scope: ScopeId,
    rename_me: L,
}

impl<'a, Val, L> DataCtx<'a, Val, L>
where
    L: Listen<Value = Val, Key = NodeId>,
{
    pub fn new(
        bucket: &'a BucketRef<'a, Val>,
        node_id: &'a NodeId,
        scope: ScopeId,
        rename_me: L,
    ) -> Self {
        Self {
            bucket,
            node_id,
            scope,
            rename_me,
        }
    }

    pub fn get(&self, key: &str) -> Option<()> {
        let path = self.bucket.get_path(key)?;
        let val = self.bucket.by_path_or_empty(path, self.scope)?;
        // subscribe to value
        L::subscribe(val, self.node_id.clone());
        Some(())
    }
}
