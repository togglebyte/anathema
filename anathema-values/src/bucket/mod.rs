use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::generation::Generation;
use crate::path::Paths;
use crate::scopes::Scopes;
use crate::slab::GenerationSlab;
use crate::{Path, ValueRef};

pub type StaticBucket<T> = RwLock<GlobalBucket<T>>;

pub struct GlobalBucket<T> {
    values: RwLock<GenerationSlab<T>>,
    scopes: RwLock<Scopes<T>>,
    paths: Paths,
}

impl<T> GlobalBucket<T> {
    fn empty(paths: Paths) -> Self {
        Self {
            values: RwLock::new(GenerationSlab::empty()),
            scopes: RwLock::new(Scopes::new()),
            paths,
        }
    }

    /// Write causes a lock
    pub fn write(&mut self) -> BucketMut<'_, T> {
        BucketMut {
            slab: self.values.write(),
            scopes: self.scopes.write(),
            paths: &mut self.paths,
        }
    }

    /// Read casues a lock.
    /// It's okay to have as many read locks as possible as long
    /// as there is no write lock
    pub fn read(&self) -> Bucket<'_, T> {
        Bucket {
            slab: self.values.read(),
            paths: &self.paths,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Bucket -
// -----------------------------------------------------------------------------
pub struct Bucket<'a, T> {
    slab: RwLockReadGuard<'a, GenerationSlab<T>>,
    paths: &'a Paths,
}

impl<'a, T> Bucket<'a, T> {
    pub(crate) fn get(&self, value_ref: ValueRef<T>) -> Option<&Generation<T>> {
        self.slab
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

unsafe impl<T> Send for Bucket<'static, T> {}
unsafe impl<T> Sync for Bucket<'static, T> {}

// -----------------------------------------------------------------------------
//   - Bucket mut -
// -----------------------------------------------------------------------------
pub struct BucketMut<'a, T> {
    slab: RwLockWriteGuard<'a, GenerationSlab<T>>,
    scopes: RwLockWriteGuard<'a, Scopes<T>>,
    paths: &'a mut Paths,
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
    pub(crate) fn insert(&mut self, path: impl Into<Path>, value: T) -> ValueRef<T> {
        let value_ref = self.slab.push(value);
        let path = path.into();
        let path_id = self.paths.get_or_insert(path);
        self.scopes.insert(path_id, value_ref, None);
        value_ref
    }

    pub(crate) fn get(&self, value_ref: ValueRef<T>) -> Option<&Generation<T>> {
        self.slab
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }

    pub(crate) fn get_mut(&mut self, value_ref: ValueRef<T>) -> Option<&mut Generation<T>> {
        self.slab
            .get_mut(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_by_path() {
        let paths = vec![Path::from("a").compose("b")];
        let mut bucket = GlobalBucket::<usize>::empty(paths.into());

        // Write a
        let value_ref = bucket.write().insert("a", 123);
        // let path = bucket.read().path_lookup("a");
        // let value_ref = bucket.read().by_path(path);

        // insert should produce paths and values and indices and magic

        // TODO: continue from here. How can we use a path without something that can
        // perform a lookup via path?
        // assert_eq!(expected, actual);
    }
}
