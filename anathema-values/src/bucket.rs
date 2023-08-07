use parking_lot::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard};

use crate::generation::Generation;
use crate::notifier::{Action, Notifier};
use crate::path::Paths;
use crate::scopes::Scopes;
use crate::slab::GenerationSlab;
use crate::values::{IntoValue, TryFromValue, TryFromValueMut};
use crate::{Path, PathId, ScopeId, Truthy, Value, ValueRef};

// TODO: Rename this, "bucket" is a bit of a stupid name here.
//       Revisit the whole thing, it makes no sense that a BucketRef can return
//       a ReadOnly since the bucket ref should be read-only by default

// -----------------------------------------------------------------------------
//   - Global bucket -
// -----------------------------------------------------------------------------
pub struct Bucket<T> {
    values: RwLock<GenerationSlab<Value<T>>>,
    scopes: RwLock<Scopes<Value<T>>>,
    paths: RwLock<Paths>,
    notifier: Notifier<T>,
}

impl<T> Bucket<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            values: RwLock::new(GenerationSlab::with_capacity(cap)),
            scopes: RwLock::new(Scopes::with_capacity(cap)),
            paths: RwLock::new(Paths::empty()),
            notifier: Notifier::new(sender),
        }
    }

    pub fn empty() -> Self {
        Self::with_capacity(0)
    }

    /// Write causes a lock
    pub fn write(&mut self) -> BucketMut<'_, T> {
        BucketMut {
            slab: self.values.write(),
            scopes: self.scopes.write(),
            paths: &self.paths,
            notifier: &self.notifier,
        }
    }

    /// Read casues a lock.
    /// It's okay to have as many read locks as possible as long
    /// as there is no write lock
    pub fn read(&self) -> BucketRef<'_, T> {
        BucketRef {
            values: &self.values,
            paths: self.paths.read(),
            scopes: &self.scopes,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Bucket ref -
// -----------------------------------------------------------------------------
pub struct BucketRef<'a, T> {
    // values: RwLockUpgradableReadGuard<'a, GenerationSlab<Value<T>>>,
    values: &'a RwLock<GenerationSlab<Value<T>>>,
    paths: RwLockReadGuard<'a, Paths>,
    scopes: &'a RwLock<Scopes<Value<T>>>,
}

impl<'a, T: Truthy> BucketRef<'a, T> {
    pub fn check_true(&self, value_ref: ValueRef<Value<T>>) -> bool {
        self.values
            .read()
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
            .map(|val| val.is_true())
            .unwrap_or(false)
    }
}

impl<'a, T> BucketRef<'a, T> {
    pub fn read(&self) -> ReadOnly<'a, T> {
        ReadOnly {
            inner: self.values.read(),
        }
    }

    pub fn by_path(
        &self,
        path_id: PathId,
        scope: impl Into<Option<ScopeId>>,
    ) -> Option<ValueRef<Value<T>>> {
        self.scopes.read().get(path_id, scope)
    }

    /// Try to get a value by path.
    /// If there is no value at a given path, insert an 
    /// empty value and return the `ValueRef` to that.
    pub fn by_path_or_empty(
        &self,
        path_id: PathId,
        scope: impl Into<Option<ScopeId>>,
    ) -> ValueRef<Value<T>> {
        let value_ref = match self.by_path(path_id, scope) {
            Some(val) => val,
            None => {
                let mut values = self.values.write();
                values.push(Value::Empty)
            }
        };

        value_ref
    }

    pub fn new_scope(&self, parent: Option<ScopeId>) -> ScopeId {
        self.scopes.write().new_scope(parent)
    }

    pub fn get_path(&self, path: impl Into<Path>) -> Option<PathId> {
        self.paths.get(&path.into())
    }

    pub fn get_path_unchecked(&self, path: impl Into<Path>) -> PathId {
        self.paths.get(&path.into()).expect("assumed path exists")
    }

    pub fn scope_value(&self, path_id: PathId, value: ValueRef<Value<T>>, scope: ScopeId) {
        self.scopes.write().insert(path_id, value, scope)
    }
}

// -----------------------------------------------------------------------------
//   - Read-only values -
// -----------------------------------------------------------------------------
pub struct ReadOnly<'a, T> {
    inner: RwLockReadGuard<'a, GenerationSlab<Value<T>>>,
}

impl<'a, T> ReadOnly<'a, T> {
    pub fn get(&self, value_ref: ValueRef<Value<T>>) -> Option<&Generation<Value<T>>> {
        self.inner
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    // TODO: reconsider this name
    pub fn getv2<V>(&self, value_ref: ValueRef<Value<T>>) -> Option<&V::Output>
    where
        V: TryFromValue<T>,
    {
        V::from_value(self.get(value_ref)?)
    }
}

// -----------------------------------------------------------------------------
//   - Bucket mut -
// -----------------------------------------------------------------------------
pub struct BucketMut<'a, T> {
    slab: RwLockWriteGuard<'a, GenerationSlab<Value<T>>>,
    scopes: RwLockWriteGuard<'a, Scopes<Value<T>>>,
    paths: &'a RwLock<Paths>,
    notifier: &'a Notifier<T>,
}

impl<'a, T> BucketMut<'a, T> {
    pub(crate) fn remove(&mut self, value_ref: ValueRef<T>) -> Generation<Value<T>> {
        self.slab.remove(value_ref.index)
    }

    pub fn push(&mut self, value: T) -> ValueRef<Value<T>> {
        self.slab.push(Value::Single(value))
    }

    pub fn insert_path(&mut self, path: impl Into<Path>) -> PathId {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        path_id
    }

    /// Insert a value at a given path.
    /// This will ensure the path will be created if it doesn't exist.
    ///
    /// This will only insert into the root scope.
    pub fn insert_at_path<V>(&mut self, path: impl Into<Path>, value: V) -> ValueRef<Value<T>>
    where
        V: IntoValue<T>,
    {
        // a
        // a.b.c

        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.insert(path_id, value)
    }

    /// Insert a value at a given path id.
    /// The value is inserted into the root scope,
    /// (A `BucketMut` should never operate on anything but the root scope.)
    pub fn insert<V>(&mut self, path_id: PathId, value: V) -> ValueRef<Value<T>>
    where
        V: IntoValue<T>,
    {
        let value = value.into_value(&mut *self);
        let value_ref = self.slab.push(value);
        self.scopes.insert(path_id, value_ref, None);
        value_ref
    }

    // TODO: rename this to something more sensible
    pub fn getv2<V>(&self, path: impl Into<Path>) -> Option<&V::Output>
    where
        V: TryFromValue<T>,
    {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.get(path_id).and_then(|v| V::from_value(v))
    }

    // TODO: rename this, you know the drill
    pub fn getv2_mut<V>(&mut self, path: impl Into<Path>) -> Option<&mut V::Output>
    where
        V: TryFromValueMut<T>,
    {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.get_mut(path_id).and_then(|v| V::from_value(v))
    }

    pub fn get(&self, path_id: PathId) -> Option<&Generation<Value<T>>> {
        let value_ref = self.scopes.get(path_id, None)?;
        self.slab
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }

    pub fn get_mut(&mut self, path_id: PathId) -> Option<&mut Generation<Value<T>>> {
        let value_ref = self.scopes.get(path_id, None)?;
        self.by_ref_mut(value_ref)
    }

    pub fn by_ref_mut(
        &mut self,
        value_ref: ValueRef<Value<T>>,
    ) -> Option<&mut Generation<Value<T>>> {
        // Notify here
        self.notifier.notify(value_ref, Action::Modified);
        self.slab
            .get_mut(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hashmap::HashMap;
    use crate::{List, Map};

    fn make_test_bucket() -> Bucket<u32> {
        let mut bucket = Bucket::empty();
        bucket.write().insert_at_path("count", 123u32);
        bucket.write().insert_at_path("len", 10);
        bucket
    }

    #[test]
    fn bucket_mut_get() {
        let mut bucket = make_test_bucket();
        let bucket = bucket.write();
        let count = bucket.getv2::<u32>("count").unwrap();
        let len = bucket.getv2::<u32>("len").unwrap();
        assert_eq!(123, *count);
        assert_eq!(10, *len);
    }

    #[test]
    fn bucket_mut_get_mut() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        *bucket.getv2_mut::<u32>("count").unwrap() = 5u32;
        let actual = bucket.getv2_mut::<u32>("count").unwrap();
        assert_eq!(5, *actual);
    }

    #[test]
    fn bucket_mut_insert_list() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        bucket.insert_at_path("list", vec![1, 2, 3]);
        let list: &List<u32> = bucket.getv2::<List<u32>>("list").unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn bucket_ref_get() {
        let bucket = make_test_bucket();
        let bucket = bucket.read();
        let count_value_ref = ValueRef::new(0, 0);
        let len_value_ref = ValueRef::new(1, 0);
        let count = bucket.get(count_value_ref).unwrap();
        let len = bucket.get(len_value_ref).unwrap();
        assert_eq!(Value::Single(123), **count);
        assert_eq!(Value::Single(10), **len);
    }
}
