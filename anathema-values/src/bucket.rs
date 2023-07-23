use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::generation::Generation;
use crate::path::Paths;
use crate::scopes::Scopes;
use crate::slab::GenerationSlab;
use crate::{Path, ValueRef};

// -----------------------------------------------------------------------------
//   - Global bucket -
// -----------------------------------------------------------------------------
pub struct Bucket<T> {
    values: RwLock<GenerationSlab<T>>,
    scopes: RwLock<Scopes<T>>,
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

    fn empty() -> Self {
        Self {
            values: RwLock::new(GenerationSlab::empty()),
            scopes: RwLock::new(Scopes::new()),
            paths: RwLock::new(Paths::new()),
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
            slab: self.values.read(),
            paths: self.paths.read(),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Bucket ref -
// -----------------------------------------------------------------------------
pub struct BucketRef<'a, T> {
    slab: RwLockReadGuard<'a, GenerationSlab<T>>,
    paths: RwLockReadGuard<'a, Paths>,
}

impl<'a, T> BucketRef<'a, T> {
    pub(crate) fn get(&self, value_ref: ValueRef<T>) -> Option<&Generation<T>> {
        self.slab
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

// -----------------------------------------------------------------------------
//   - Bucket mut -
// -----------------------------------------------------------------------------
pub struct BucketMut<'a, T> {
    slab: RwLockWriteGuard<'a, GenerationSlab<T>>,
    scopes: RwLockWriteGuard<'a, Scopes<T>>,
    paths: &'a RwLock<Paths>,
}

impl<'a, T> BucketMut<'a, T> {
    pub(crate) fn remove(&mut self, value_ref: ValueRef<T>) -> Generation<T> {
        self.slab.remove(value_ref.index)
    }

    pub(crate) fn push(&mut self, value: T) -> ValueRef<T> {
        self.slab.push(value)
    }

    // Insert a value at a given path.
    // This will ensure the path will be created if it doesn't exist.
    pub fn insert(&mut self, path: impl Into<Path>, value: T) -> ValueRef<T> {
        let value_ref = self.slab.push(value);
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.scopes.insert(path_id, value_ref, None);
        value_ref
    }

    pub fn get(&self, path: impl Into<Path>) -> Option<&Generation<T>> {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        let value_ref = self.scopes.get(path_id, None)?;
        self.slab
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }

    pub fn get_mut(&mut self, path: impl Into<Path>) -> Option<&mut Generation<T>> {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        let value_ref = self.scopes.get(path_id, None)?;
        self.slab
            .get_mut(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

#[cfg(test)]
mod test {
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
        let mut bucket = bucket.write();
        let count = bucket.get("count").unwrap();
        let len = bucket.get("len").unwrap();
        assert_eq!(123, **count);
        assert_eq!(10, **len);
    }

    #[test]
    fn bucket_mut_get_mut() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
        **bucket.get_mut("count").unwrap() = 5u32;
        let actual = bucket.get_mut("count").unwrap();
        let expected = 5u32;
        assert_eq!(expected, **actual);
    }

    #[test]
    fn bucket_mut_insert_map() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.write();
    }

    #[test]
    fn bucket_ref_get() {
        let mut bucket = make_test_bucket();
        let mut bucket = bucket.read();
        let count_value_ref = ValueRef::new(0, 0);
        let len_value_ref = ValueRef::new(1, 0);
        let count = bucket.get(count_value_ref).unwrap();
        let len = bucket.get(len_value_ref).unwrap();
        assert_eq!(123, **count);
        assert_eq!(10, **len);
    }
}
