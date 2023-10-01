use std::fmt::Debug;

use super::*;
use crate::hashmap::HashMap;
use crate::Path;

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, StateValue<T>>,
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
        }
    }

    pub fn lookup(&self, path: &Path, node_id: Option<&NodeId>) -> Option<&StateValue<T>>
    where
        for<'a> ValueRef<'a>: From<&'a T>,
    {
        match path {
            Path::Key(key) => self.inner.get(key),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Mappy -
// -----------------------------------------------------------------------------
pub trait Mappy: Debug {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>;
}

impl<T: Debug> Mappy for Map<T>
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
    fn create_map() {
        let state = TestState::new();
        let path = Path::from("generic_map");
        let path = path.compose("second");
        let x = state.get(&path, None);
        panic!("{x:#?}");
    }
}
