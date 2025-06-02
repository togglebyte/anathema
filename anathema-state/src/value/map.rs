use std::collections::HashMap;

use super::{Shared, Type, Unique, Value};
use crate::PendingValue;
use crate::states::{AnyMap, AnyState};
use crate::store::values::{get_unique, try_make_shared};

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, Value<T>>,
}

impl<T: AnyState> Map<T> {
    pub fn empty() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<&Value<T>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<T>> {
        self.inner.get_mut(key)
    }

    /// Insert a value into the `Map`.
    /// The value will be wrapped in a `Value<T>` so it's not advisable to insert pre-wrapped
    /// value.
    pub fn insert(&mut self, map_key: impl Into<String>, value: T) {
        let value = value.into();
        let map_key = map_key.into();
        self.inner.insert(map_key, value);
    }

    /// Remove a value from the map.
    pub fn remove(&mut self, map_key: &str) -> Option<Value<T>> {
        self.inner.remove(map_key)
    }
}

impl<T: AnyState> Default for Map<T> {
    fn default() -> Self {
        Self { inner: HashMap::new() }
    }
}

/// A `Map` of values with strings as keys.
/// ```
/// # use anathema_state::Map;
/// let mut map = Map::empty();
/// map.insert("key", 123);
/// ```
impl<T: AnyState> Value<Map<T>> {
    pub fn empty() -> Self {
        let map = Map { inner: HashMap::new() };
        Value::new(map)
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<Shared<'_, T>> {
        let map = &*self.to_ref();
        let value = map.get(key.as_ref())?;
        let key = value.key;

        let (key, value) = try_make_shared(key.owned())?;
        let shared = Shared::new(key, value);
        Some(shared)
    }

    pub fn get_mut<'a>(&'a mut self, key: impl AsRef<str>) -> Option<Unique<'a, T>> {
        let map = &*self.to_ref();
        let value = map.get(key.as_ref())?;

        let key = value.key;
        let value = Unique {
            value: Some(get_unique(key.owned())),
            key,
            _p: std::marker::PhantomData,
        };
        Some(value)
    }
}

impl<T: AnyState + 'static> AnyMap for Map<T> {
    fn lookup(&self, key: &str) -> Option<PendingValue> {
        self.get(key).map(|val| val.reference())
    }
}

impl<T: AnyState + 'static> AnyState for Map<T> {
    fn type_info(&self) -> Type {
        Type::Map
    }

    fn to_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any_map(&self) -> Option<&dyn AnyMap> {
        Some(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert() {
        let mut map = Map::empty();
        map.insert("a", 1);
        map.insert("b", 2);

        let val = map.get("a").unwrap().to_ref();
        assert_eq!(*val, 1);

        let val = map.get("b").unwrap().to_ref();
        assert_eq!(*val, 2);
    }

    struct DM(usize);
    impl crate::State for DM {
        fn type_info(&self) -> Type {
            Type::Unit
        }
    }
    impl Drop for DM {
        fn drop(&mut self) {
            eprintln!("- drop: {}", self.0);
        }
    }
    struct DMRef<'a>(&'a DM);
    impl Drop for DMRef<'_> {
        fn drop(&mut self) {
            eprintln!("- drop ref: {}", self.0.0);
        }
    }

    #[test]
    fn remove() {
        let mut map = Map::empty();
        map.insert("a", DM(1));
        // map.insert("b", 1i32);
        let value_ref = map.get("a").as_ref().unwrap();
        // // let value_mut = map.get_mut("b").unwrap();
        // // assert!(*value_ref == 1);
        // // assert!(*value_mut == 1);
        eprintln!("going to remove");

        map.remove("a");
        // eprintln!("hi");
        // // drop(value_ref);
        // // assert!(value_ref.value::<i32>().is_none());

        panic!()
    }
}
