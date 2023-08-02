use std::fmt::{self, Debug};
use crate::hashmap::IntMap;
use crate::{ValueRef, ValueV2, PathId};

#[derive(PartialEq)]
pub struct Map<T>(IntMap<ValueRef<ValueV2<T>>>);

impl<T> Map<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> () {
        self.0.iter();
    }

    /// Insert a value at a path.
    /// ```
    /// # use std::collections::HashMap;
    /// # use anathema_values::{Map, BucketMut};
    /// # fn run(bucket: &mut BucketMut<'_, String>) -> Option<()> {
    /// let map = HashMap::<&str, String>::from_iter(vec![("tea", "earl grey".to_string()),]);
    /// bucket.insert_at_path("themap", map);
    ///
    /// let sugar = bucket.insert_path("sugar");
    /// let value_ref = bucket.insert(sugar, "no, thank you".to_string());
    ///
    /// let map = bucket.getv2_mut::<Map<_>>("themap")?;
    /// map.insert(sugar, value_ref);
    /// # Some(())
    /// # }
    /// ```
    pub fn insert(&mut self, path: PathId, value: ValueRef<ValueV2<T>>) {
        self.0.insert(path.0, value);
    }
}


impl<T> From<IntMap<ValueRef<ValueV2<T>>>> for Map<T> {
    fn from(v: IntMap<ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}

impl<T> Debug for Map<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Map")
            .field(&self.0)
            .finish()
    }
}

impl<T> Clone for Map<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

