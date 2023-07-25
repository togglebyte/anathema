use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::generation::Generation;
use crate::path::Paths;
use crate::scopes::Scopes;
use crate::slab::GenerationSlab;
use crate::values2::{IntoValue, TryFromValue};
use crate::{Path, PathId, ScopeId, ValueRef, ValueV2};

// -----------------------------------------------------------------------------
//   - Global bucket -
// -----------------------------------------------------------------------------
pub struct Bucket<T> {
    values: RwLock<GenerationSlab<ValueV2<T>>>,
    scopes: RwLock<Scopes<ValueV2<T>>>,
    paths: RwLock<Paths>,
}

impl<T> Bucket<T> {
    pub fn new(paths: Paths) -> Self {
        Self {
            values: RwLock::new(GenerationSlab::empty()),
            scopes: RwLock::new(Scopes::new()),
            paths: RwLock::new(paths),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            values: RwLock::new(GenerationSlab::with_capacity(cap)),
            scopes: RwLock::new(Scopes::with_capacity(cap)),
            paths: RwLock::new(Paths::empty()),
        }
    }

    pub fn empty() -> Self {
        Self {
            values: RwLock::new(GenerationSlab::empty()),
            scopes: RwLock::new(Scopes::new()),
            paths: RwLock::new(Paths::empty()),
        }
    }

    /// Write causes a lock
    pub fn write(&mut self) -> BucketMut<'_, T> {
        BucketMut {
            slab: self.values.write(),
            scopes: self.scopes.write(),
            paths: &self.paths,
        }
    }

    /// Read casues a lock.
    /// It's okay to have as many read locks as possible as long
    /// as there is no write lock
    pub fn read(&self) -> BucketRef<'_, T> {
        BucketRef {
            values: self.values.read(),
            paths: self.paths.read(),
            scopes: &self.scopes,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Bucket ref -
// -----------------------------------------------------------------------------
pub struct BucketRef<'a, T> {
    values: RwLockReadGuard<'a, GenerationSlab<ValueV2<T>>>,
    paths: RwLockReadGuard<'a, Paths>,
    scopes: &'a RwLock<Scopes<ValueV2<T>>>,
}

impl<'a, T> BucketRef<'a, T> {
    pub fn get(&self, value_ref: ValueRef<ValueV2<T>>) -> Option<&Generation<ValueV2<T>>> {
        self.values
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    pub fn getv2<V>(&self, value_ref: ValueRef<ValueV2<T>>) -> Option<&V::Output> 
        where V: TryFromValue<T>
    {
        V::from_value(self.get(value_ref)?)
    }

    pub fn by_path(&self, path_id: PathId, scope: impl Into<Option<ScopeId>>) -> Option<&Generation<ValueV2<T>>> {
        let value_ref = self.scopes.read().get(path_id, scope)?;
        self.get(value_ref)
    }

    pub fn by_pathv2<V>(&self, path_id: PathId, scope: impl Into<Option<ScopeId>>) -> Option<&V::Output> 
        where V: TryFromValue<T>
    {
        V::from_value(self.by_path(path_id, scope)?)
    }

    pub fn new_scope(&self, parent: Option<ScopeId>) -> ScopeId {
        self.scopes.write().new_scope(parent)
    }

    pub fn get_path_unchecked(&self, path: impl Into<Path>) -> PathId {
        self.paths.get(&path.into()).expect("assumed path exists")
    }
}

// -----------------------------------------------------------------------------
//   - Bucket mut -
// -----------------------------------------------------------------------------
pub struct BucketMut<'a, T> {
    slab: RwLockWriteGuard<'a, GenerationSlab<ValueV2<T>>>,
    scopes: RwLockWriteGuard<'a, Scopes<ValueV2<T>>>,
    paths: &'a RwLock<Paths>,
}

impl<'a, T> BucketMut<'a, T> {
    pub(crate) fn remove(&mut self, value_ref: ValueRef<T>) -> Generation<ValueV2<T>> {
        self.slab.remove(value_ref.index)
    }

    pub(crate) fn push(&mut self, value: T) -> ValueRef<ValueV2<T>> {
        self.slab.push(ValueV2::Single(value))
    }

    pub(crate) fn insert_path(&mut self, path: impl Into<Path>) -> PathId {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        path_id
    }

    // Insert a value at a given path.
    // This will ensure the path will be created if it doesn't exist.
    pub fn insert<V>(&mut self, path: impl Into<Path>, value: V) -> ValueRef<ValueV2<T>>
    where
        V: IntoValue<T>,
    {
        let value = value.into_value(&mut *self);
        let value_ref = self.slab.push(value);
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.scopes.insert(path_id, value_ref, None);
        value_ref
    }

    pub fn bulk_insert<P: Into<Path>>(&mut self, data: Vec<(P, T)>) {
        let mut paths = self.paths.write();

        for (path, value) in data {
            let value = value.into_value(&mut *self);
            let value_ref = self.slab.push(value);
            let path = path.into();
            let path_id = paths.get_or_insert(path);
            self.scopes.insert(path_id, value_ref, None);
        }
    }

    pub fn by_ref(&self, value_ref: ValueRef<T>) -> Option<&Generation<ValueV2<T>>> {
        self.slab
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    pub fn by_ref_mut(&mut self, value_ref: ValueRef<T>) -> Option<&mut Generation<ValueV2<T>>> {
        self.slab
            .get_mut(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    pub fn getv2<V>(&self, path: impl Into<Path>) -> Option<&V::Output> 
        where V: TryFromValue<T>
    {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        let value_ref = self.scopes.get(path_id, None)?;
        let val: &ValueV2<T> = &*self
            .slab
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))?;

        V::from_value(val)
    }

    pub fn get(&self, path: impl Into<Path>) -> Option<&Generation<ValueV2<T>>> {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        let value_ref = self.scopes.get(path_id, None)?;
        self.slab
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    pub fn get_mut(&mut self, path: impl Into<Path>) -> Option<&mut Generation<ValueV2<T>>> {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        let value_ref = self.scopes.get(path_id, None)?;
        self.slab
            .get_mut(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }
}

#[cfg(test)]
mod test {
    use crate::{Map, hashmap::HashMap, List};

    use super::*;

    fn make_test_bucket() -> Bucket<u32> {
        let mut bucket = Bucket::empty();
        bucket.write().insert("count", 123);
        bucket.write().insert("len", 10);
        bucket
    }

    #[test]
    fn bucket_mut_get() {
        let mut bucket = make_test_bucket();
        let bucket = bucket.write();
        let count = bucket.get("count").unwrap();
        let len = bucket.get("len").unwrap();
        assert_eq!(ValueV2::Single(123), **count);
        assert_eq!(ValueV2::Single(10), **len);
    }

    #[test]
    fn bucket_mut_get_mut() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        **bucket.get_mut("count").unwrap() = ValueV2::Single(5u32);
        let actual = bucket.get_mut("count").unwrap();
        assert_eq!(ValueV2::Single(5), **actual);
    }

    #[test]
    fn bucket_mut_insert_list() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        bucket.insert("list", vec![1, 2, 3]);
        let list: &List<u32> = bucket.getv2::<List<u32>>("list").unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn bucket_mut_insert_map() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        let hm = HashMap::from([("a", 1), ("b", 2)]);
        bucket.insert("map", hm);
        let map: &Map<u32> = bucket.getv2::<Map<u32>>("map").unwrap();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn bucket_ref_get() {
        let bucket = make_test_bucket();
        let bucket = bucket.read();
        let count_value_ref = ValueRef::new(0, 0);
        let len_value_ref = ValueRef::new(1, 0);
        let count = bucket.get(count_value_ref).unwrap();
        let len = bucket.get(len_value_ref).unwrap();
        assert_eq!(ValueV2::Single(123), **count);
        assert_eq!(ValueV2::Single(10), **len);
    }
}
