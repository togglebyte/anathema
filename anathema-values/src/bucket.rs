use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub type StaticBucket<T> = OnceLock<RwLock<GlobalBucket<T>>>;

fn new_bucket<T>() -> RwLock<GlobalBucket<T>> {
    RwLock::new(GlobalBucket(Vec::new()))
}

#[derive(Default)]
pub struct GlobalBucket<T>(Vec<T>);

impl<T> GlobalBucket<T> {
    // Worst case scenario: dead lock
    pub(crate) fn write(glob: &'static StaticBucket<T>) -> BucketMut<'static, T> {
        BucketMut(glob.get_or_init(new_bucket).write())
    }

    pub fn read(glob: &'static StaticBucket<T>) -> Bucket<'static, T> {
        Bucket(glob.get_or_init(new_bucket).read())
    }

    pub(crate) fn push(&mut self, value: T) -> usize {
        let value_id = self.0.len();
        self.0.push(value);
        value_id
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)
    }
}

pub struct Bucket<'a, T>(RwLockReadGuard<'a, GlobalBucket<T>>);

unsafe impl<T> Send for Bucket<'static, T> {}
unsafe impl<T> Sync for Bucket<'static, T> {}

impl<'a, T> Deref for Bucket<'a, T> {
    type Target = RwLockReadGuard<'a, GlobalBucket<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct BucketMut<'a, T>(RwLockWriteGuard<'a, GlobalBucket<T>>);

impl<'a, T> Deref for BucketMut<'a, T> {
    type Target = RwLockWriteGuard<'a, GlobalBucket<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for BucketMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
