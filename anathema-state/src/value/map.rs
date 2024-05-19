use std::collections::HashMap;
use std::rc::Rc;

use super::Value;
use crate::{CommonVal, Path, PendingValue, State, Subscriber, ValueRef};

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<Rc<str>, Value<T>>,
}

impl<T: 'static + State> Map<T> {
    pub fn empty() -> Value<Self> {
        Value::<Self>::empty()
    }

    pub fn get(&self, key: &str) -> Option<&Value<T>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<T>> {
        self.inner.get_mut(key)
    }
}

impl<T: 'static + State> Value<Map<T>> {
    pub fn empty() -> Self {
        let map = Map { inner: HashMap::new() };
        Value::new(map)
    }

    /// Insert a value into the `Map`.
    /// The value will be wrapped in a `Value<T>` so it's not advisable to insert pre-wrapped
    /// value.
    pub fn insert(&mut self, map_key: impl Into<Rc<str>>, value: impl Into<Value<T>>) {
        let map_key = map_key.into();
        let map = &mut *self.to_mut();
        let value = value.into();
        map.inner.insert(map_key, value);
    }

    pub fn remove(&mut self, map_key: &str) -> Option<Value<T>> {
        let _key = self.key;
        let map = &mut *self.to_mut();
        map.inner.remove(map_key)
    }
}

impl<T: 'static + State> State for Map<T> {
    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        let Path::Key(k) = path else { return None };
        let value = self.inner.get(k)?;
        Some(value.value_ref(sub))
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        let Path::Key(k) = path else { return None };
        let value = self.inner.get(k)?;
        Some(value.to_pending())
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
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

        let r = map.to_ref();

        let val = *r.get("a").unwrap().to_ref();
        assert_eq!(val, 1);

        let val = *r.get("b").unwrap().to_ref();
        assert_eq!(val, 2);
    }

    #[test]
    fn remove() {
        let mut map = Map::empty();
        map.insert("a", 1i32);
        let value_ref = map.to_ref().state_get("a".into(), Subscriber::ZERO).unwrap();
        map.remove("a");
        assert!(value_ref.value::<i32>().is_none());
    }
}
