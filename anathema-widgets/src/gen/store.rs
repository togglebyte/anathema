use std::borrow::{Cow};
use std::collections::HashMap;



use super::ValueRef;
use crate::{DataCtx, Value};

// -----------------------------------------------------------------------------
//   - Layout -
// -----------------------------------------------------------------------------
#[derive(Debug)]
// TODO: rename this to something less stupid
pub struct Layout<'parent>(HashMap<Cow<'parent, str>, ValueRef<'parent>>);

impl<'parent> Layout<'parent> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: Cow<'parent, str>, value: ValueRef<'parent>) {
        self.0.insert(key, value);
    }

    pub fn by_key(&self, key: &str) -> Option<&ValueRef<'parent>> {
        self.0.get(key)
    }

    pub fn set(&mut self, key: Cow<'parent, str>, val: ValueRef<'parent>) {
        self.0.insert(key, val);
    }
}

// -----------------------------------------------------------------------------
//   - Store -
// -----------------------------------------------------------------------------
#[derive(Debug)]
// TODO: Rename this. It's a value store, but it needs a better name
pub struct Store<'parent> {
    root: &'parent DataCtx,
    parent: Option<&'parent Store<'parent>>,
    pub inner: Layout<'parent>,
}

impl<'parent> Store<'parent> {
    pub fn new(root: &'parent DataCtx) -> Self {
        Self {
            root,
            parent: None,
            inner: Layout::new(),
        }
    }

    pub fn next(&self) -> Store<'_> {
        Store {
            root: self.root,
            parent: Some(&self),
            inner: Layout::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.inner
            .by_key(key)
            .and_then(|v| v.value())
            .or_else(|| self.parent.and_then(|p| p.get(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub fn get_borrowed(&self, key: &str) -> Option<&'parent Value> {
        self.inner
            .by_key(key)
            .and_then(|v| v.borrowed())
            .or_else(|| self.parent.and_then(|p| p.get(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub fn set(&mut self, key: Cow<'parent, str>, val: ValueRef<'parent>) {
        self.inner.set(key, val);
    }
}

// #[cfg(test)]
// mod test {
//     use std::borrow::Cow;

//     use super::*;

//     fn root() -> HashMap<String, usize> {
//         let mut root = HashMap::new();
//         root.insert("key".to_string(), 1);
//         root
//     }

//     #[test]
//     fn get_nested_values() {
//         let root = root();
//         let mut values = Store::<_, HashMap<Cow<'_, str>, ValueRef<'_, usize>>>::new(&root);
//         assert_eq!(values.get("key").unwrap(), &1);

//         let mut values = values.next();
//         values.set("key2".into(), ValueRef::Owned(2));
//         let value = values.get("key2").unwrap();
//         assert_eq!(*value, 2);

//         let value_1 = values.get("key").unwrap();
//         let mut values = values.next();
//         values.set("key2".into(), ValueRef::Borrowed(value_1));
//         assert_eq!(*values.get("key2").unwrap(), 1);
//     }

//     #[test]
//     fn root_ctx() {
//         struct Root;

//         impl Get for Root {
//             type Key = usize;
//             type Value<'a> = ();

//             fn by_key<'a, Q>(&self, key: &Q) -> Option<&Self::Value<'a>>
//             where
//                 Self::Key: Borrow<Q>,
//                 Q: Hash + Eq + ?Sized,
//             {
//                 Some(&())
//             }
//         }

//         let root = Root;
//         let values = Store::<_, ()>::new(&root);
//     }
// }
