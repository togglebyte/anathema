use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::generation::Generation;
use crate::slab::Slab;
use crate::{Path, ValueRef};

pub type StaticBucket<T> = RwLock<GlobalBucket<T>>;

pub struct GlobalBucket<T> {
    values: RwLock<Slab<T>>,
    paths: Vec<Path>,
}

impl<T> GlobalBucket<T> {
    fn empty(paths: Vec<Path>) -> Self {
        Self {
            values: RwLock::new(Slab::empty()),
            paths,
        }
    }

    /// Write causes a lock
    pub fn write(&self) -> BucketMut<'_, T> {
        BucketMut(self.values.write())
    }

    /// Read casues a lock.
    /// It's okay to have as many read locks as possible as long
    /// as there is no write lock
    pub fn read(&self) -> Bucket<'_, T> {
        Bucket(self.values.read(), &self.paths)
    }
}

// -----------------------------------------------------------------------------
//   - Bucket -
// -----------------------------------------------------------------------------
pub struct Bucket<'a, T>(RwLockReadGuard<'a, Slab<T>>, &'a [Path]);

impl<'a, T> Bucket<'a, T> {
    pub(crate) fn get(&self, value_ref: ValueRef<T>) -> Option<&Generation<T>> {
        self.0
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

unsafe impl<T> Send for Bucket<'static, T> {}
unsafe impl<T> Sync for Bucket<'static, T> {}

// -----------------------------------------------------------------------------
//   - Bucket mut -
// -----------------------------------------------------------------------------
pub struct BucketMut<'a, T>(RwLockWriteGuard<'a, Slab<T>>);

impl<'a, T> BucketMut<'a, T>{
    pub(crate) fn remove(&mut self, value_ref: ValueRef<T>) -> Generation<T> {
        self.0.remove(value_ref.index)
    }

    pub(crate) fn push(&mut self, value: T) -> usize {
        self.0.push(value)
    }

    pub(crate) fn get(&self, value_ref: ValueRef<T>) -> Option<&Generation<T>> {
        self.0
            .get(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }

    pub(crate) fn get_mut(&mut self, value_ref: ValueRef<T>) -> Option<&mut Generation<T>> {
        self.0
            .get_mut(value_ref.index)
            .filter(|val| val.comp_gen(value_ref.gen))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_by_path() {
        // TODO: continue from here. How can we use a path without something that can 
        // perform a lookup via path?
        let paths = vec![Path::from("a")];
        let bucket = GlobalBucket::<usize>::empty(paths);
        // assert_eq!(expected, actual);
    }
}
