use std::collections::HashMap;
use std::rc::Rc;

use super::{Type, Value};
use crate::states::{AnyMap, AnyValue};
use crate::{CommonVal, Path, PendingValue, State, Subscriber, ValueRef};

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, Value<T>>,
}

impl<T: AnyValue> Map<T> {
    pub fn empty() -> Self {
        Self { inner: HashMap::new() }
    }

    // TODO if this has to go back into the Value<Self> then remove this function
    // along with having the `empty` funcition return Value<Self> instead of Self
    pub fn insert(&mut self, map_key: impl Into<String>, value: impl Into<Value<T>>) {
        let map_key = map_key.into();
        // let map = &mut *self.to_mut();
        let value = value.into();
        self.inner.insert(map_key, value);
    }

    pub fn get(&self, key: std::borrow::Cow<'_, str>) -> Option<&Value<T>> {
        self.inner.get(&*key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<T>> {
        self.inner.get_mut(key)
    }
}

impl<T: 'static + AnyValue> Value<Map<T>> {
    // pub fn empty() -> Self {
    //     let map = Map { inner: HashMap::new() };
    //     Value::new(map)
    // }

    /// Insert a value into the `Map`.
    /// The value will be wrapped in a `Value<T>` so it's not advisable to insert pre-wrapped
    /// value.
    pub fn insert(&mut self, map_key: impl Into<String>, value: impl Into<Value<T>>) {
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

impl<T: AnyValue> AnyMap for Map<T> {
    fn lookup(&self, key: std::borrow::Cow<'_, str>) -> Option<PendingValue> {
        self.get(key).map(|val| val.reference())
    }
}

impl<T: AnyValue> AnyValue for Map<T> {
    fn type_id(&self) -> Type {
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

// impl<T: 'static + AnyValue> State for Map<T> {
//     fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
//         let Path::Key(k) = path else { return None };
//         let value = self.inner.get(k)?;
//         Some(value.value_ref(sub))
//     }

//     fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
//         let Path::Key(k) = path else { return None };
//         let value = self.inner.get(k)?;
//         Some(value.reference())
//     }

//     fn to_common(&self) -> Option<CommonVal> {
//         None
//     }
// }

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
