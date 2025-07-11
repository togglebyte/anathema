use std::collections::HashMap;

use crate::slab::{Slab, SlabIndex};

pub struct IndexMap<I, K> {
    slab: Slab<I, K>,
    map: HashMap<K, I>,
}

impl<I, K> IndexMap<I, K>
where
    I: SlabIndex,
    K: Eq,
    K: std::hash::Hash,
    K: Clone,
{
    /// Create an empty index map
    pub fn empty() -> Self {
        Self {
            slab: Slab::empty(),
            map: HashMap::new(),
        }
    }

    /// Insert a value into the map and index and return the index to the value
    pub fn insert(&mut self, key: K) -> I {
        *self.map.entry(key.clone()).or_insert_with(|| self.slab.insert(key))
    }

    /// Get a value from the index
    pub fn get(&self, index: I) -> Option<&K> {
        self.slab.get(index)
    }
}
