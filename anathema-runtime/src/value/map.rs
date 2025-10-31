use std::collections::HashMap;

use super::{AnonValue, Value};
use crate::states::{AnyMap, State};
use crate::value::{Type, ValueMut, ValueRef};

#[derive(Debug)]
pub struct Map<V> {
    inner: HashMap<String, Value<V>>,
}

impl<V: State> Map<V> {
    pub fn empty() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<&Value<V>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<V>> {
        self.inner.get_mut(key)
    }

    /// Insert a value into the `Map`.
    /// The value will be wrapped in a `Value<T>` so it's not advisable to insert pre-wrapped
    /// value.
    pub fn insert(&mut self, map_key: impl Into<String>, value: V) {
        let value = value.into();
        let map_key = map_key.into();
        self.inner.insert(map_key, value);
    }

    /// Remove a value from the map.
    pub fn remove(&mut self, map_key: &str) -> Option<Value<V>> {
        self.inner.remove(map_key)
    }

    /// Returns true if the map is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<V: State> Default for Map<V> {
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
impl<V: State> Value<Map<V>> {
    pub fn empty() -> Self {
        let map = Map { inner: HashMap::new() };
        Value::new(map)
    }
}

impl<V: State> AnyMap for Map<V> {
    fn lookup(&self, key: &str) -> Option<AnonValue> {
        self.get(key).map(|val| val.reference())
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<V: State> State for Map<V> {
    fn type_info(&self) -> Type {
        Type::Map
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

    #[derive(Debug, PartialEq)]
    struct DM(usize);

    impl crate::State for DM {
        fn type_info(&self) -> Type {
            Type::Unit
        }
    }
    impl Drop for DM {
        fn drop(&mut self) {
            // eprintln!("- drop: {}", self.0);
        }
    }

    #[test]
    fn remove() {
        let mut map = Map::empty();
        map.insert("a", DM(1));
        assert!(map.remove("a").is_some());
        assert!(map.is_empty());
    }
}
