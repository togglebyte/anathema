use std::cell::{Ref, RefCell};
use std::sync::Arc;

use parking_lot::{
    Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard,
};

use super::Values2;
use crate::generation::Generation;
use crate::hashmap::{HashMap, IntMap};
use crate::notifier::{Action, Notifier};
use crate::path::Paths;
use crate::scopes::{ScopeValue, Scopes};
use crate::slab::GenerationSlab;
use crate::values::{IntoValue, TryFromValue, TryFromValueMut};
use crate::{Container, Path, PathId, ScopeId, Truthy, ValueRef};

// This struct has to be able to store PathId and values
//
// * Global Values and global Paths
// * Inserting a key + value translates to PathId and ValueRef<T>
// * Getting / Reading in the runtime is done without scopes, so that happens
// by doing a lookup of key to PathId and then fetch ValueRef<T> by PathId, finally
// fetch value by ValueRef
// * Widget evaluation process however needs scopes (because for-loops that's why!)
#[derive(Debug, Clone)]
pub struct Map<T> {
    inner: RefCell<IntMap<ValueRef<Container<T>>>>,
}

impl<T> Map<T> {
    pub(crate) fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub(super) fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn insert(&self, path_id: PathId, value_ref: ValueRef<Container<T>>) {
        self.inner.borrow_mut().insert(path_id.0, value_ref);
    }

    pub fn get(&self, path_id: PathId) -> Option<ValueRef<Container<T>>> {
        // let path = key.into();
        // let path_id = self.paths.lock().get(&path)?;
        self.inner.borrow().get(&path_id.0).copied()
    }
}

impl<T> From<Map<T>> for Container<T> {
    fn from(value: Map<T>) -> Self {
        Self::Map(value)
    }
}

// -----------------------------------------------------------------------------
//   - Map reference -
// -----------------------------------------------------------------------------
pub struct MapRef<'a, T> {
    map: Ref<'a, Map<T>>,
    paths: &'a Paths,
    values: &'a Values2<T>,
}

impl<'a, T> MapRef<'a, T> {
    pub(super) fn new(map: Ref<'a, Map<T>>, paths: &'a Paths, values: &'a Values2<T>) -> Self {
        Self { map, paths, values }
    }

    pub fn new_map(&self, key: impl Into<Path>) -> MapRef<'_, T> {
        let path = key.into();
        let path_id = self.paths.get_or_insert(path);
        let mut map = Container::Map(Map::new());
        let value_ref = self.values.push(map);
        self.map.insert(path_id, value_ref);
        MapRef::new(self.values.get_map(value_ref), self.paths, self.values)
    }

    pub fn insert<V>(&self, path: impl Into<Path>, value: V)
    where
        V: Into<Container<T>>,
    {
        let path = path.into();
        let path_id = self.paths.get_or_insert(path);
        let value = value.into();
        let value_ref = self.values.push(value);
    }

    pub fn get(&self, path: impl Into<Path>) -> Option<Ref<'_, Container<T>>> {
        let path = path.into();
        let path_id = self.paths.get(&path)?;
        let value_ref = self.map.get(path_id)?;
        self.values.get(value_ref)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::store::Store2;

    #[test]
    fn omgwhatsgoingon() {
        let mut store = Store2::<String>::new();
        let values = store.values();
        let root_ref = values.root_ref();

        // pretend update context
        {
            let fin = root_ref.new_map("fin");
        }
    }
}
