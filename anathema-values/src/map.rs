use std::fmt::Debug;

use super::*;
use crate::hashmap::HashMap;
use crate::Path;

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, StateValue<T>>,
    // subscribers: RefCell<Vec<NodeId>>,
}

impl<T> Map<T> {
    pub fn empty() -> Self {
        Self::new::<String>(HashMap::new())
    }

    pub fn new<K: Into<String>>(inner: impl IntoIterator<Item = (K, T)>) -> Self {
        let inner = inner
            .into_iter()
            .map(|(k, v)| (k.into(), StateValue::new(v)));
        Self {
            inner: HashMap::from_iter(inner),
            // subscribers: RefCell::new(vec![]),
        }
    }

    pub fn lookup(&self, path: &Path, _node_id: Option<&NodeId>) -> Option<&StateValue<T>>
    where
        for<'a> ValueRef<'a>: From<&'a T>,
    {
        let Path::Key(key) = path else { return None };
        self.inner.get(key)
    }
}

impl<T: Debug> Collection for HashMap<String, StateValue<T>>
where
    for<'a> ValueRef<'a>: From<&'a T>,
{
    fn get(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match path {
            Path::Key(key) => {
                let value = self.get(key)?;
                if let Some(node_id) = node_id.cloned() {
                    value.subscribe(node_id);
                }
                Some((&value.inner).into())
            }
            _ => None,
        }
    }
}

impl<T: Debug> Collection for Map<T>
where
    for<'a> ValueRef<'a>: From<&'a T>,
{
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match key {
            Path::Key(_) => {
                let value = self.lookup(key, node_id)?;
                if let Some(node_id) = node_id.cloned() {
                    value.subscribe(node_id);
                }
                Some((&value.inner).into())
            }
            Path::Composite(lhs, rhs) => {
                let map = self
                    .lookup(&**lhs, node_id)
                    .map(|value| (&value.inner).into())?;

                match map {
                    ValueRef::Map(map) => map.get(rhs, node_id),
                    _ => None,
                }
            }
            Path::Index(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::TestState;

    #[test]
    fn access_map() {
        let state = TestState::new();
        let path = Path::from("generic_map").compose("inner").compose("second");
        let ValueRef::Owned(Owned::Num(x)) = state.get(&path, None).unwrap() else { panic!() };
        assert_eq!(x.to_i128(), 2);
    }
}
