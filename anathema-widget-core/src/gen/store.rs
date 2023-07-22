use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use super::ValueRef;
use crate::contexts::DataCtx;

// -----------------------------------------------------------------------------
//   - Layout -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct ScopedValues<'parent>(HashMap<Cow<'parent, str>, ValueRef<'parent>>);

impl<'parent> ScopedValues<'parent> {
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
//   - Values -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Values<'parent> {
    pub(crate) root: &'parent DataCtx,
    parent: Option<&'parent Values<'parent>>,
    pub inner: ScopedValues<'parent>,
}

impl<'parent> Values<'parent> {
    pub fn text_to_string(&self, text: &'parent TextPath) -> Cow<'parent, str> {
        match text {
            TextPath::Fragments(fragments) => {
                let mut output = String::new();
                for fragment in fragments {
                    match fragment {
                        Fragment::String(s) => output.push_str(s),
                        Fragment::Data(path) => {
                            let _ = path
                                .lookup_value(self)
                                .map(|val| write!(&mut output, "{val}"));
                        }
                    }
                }
                Cow::Owned(output)
            }
            TextPath::String(s) => Cow::from(s),
        }
    }

    pub fn new(root: &'parent DataCtx) -> Self {
        Self {
            root,
            parent: None,
            inner: ScopedValues::new(),
        }
    }

    pub fn next(&self) -> Values<'_> {
        Values {
            root: self.root,
            parent: Some(&self),
            inner: ScopedValues::new(),
        }
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.get_value(key)?.try_into().ok()
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.inner
            .by_key(key)
            .and_then(|v| v.value())
            .or_else(|| self.parent.and_then(|p| p.get_value(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub fn get_borrowed_value(&self, key: &str) -> Option<&'parent Value> {
        self.inner
            .by_key(key)
            .and_then(|v| v.borrowed())
            .or_else(|| self.parent.and_then(|p| p.get_value(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub fn set(&mut self, key: Cow<'parent, str>, val: ValueRef<'parent>) {
        self.inner.set(key, val);
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use super::*;

    fn root() -> DataCtx {
        let mut root = DataCtx::default();
        root.insert("key".to_string(), 1);
        root
    }

    #[test]
    fn get_nested_values() {
        let root = root();
        let mut values = Values::new(&root);
        assert_eq!(*values.get_ref::<i64>("key").unwrap(), 1);

        let mut values = values.next();
        values.set("key2".into(), ValueRef::Owned(2.into()));
        let value = values.get_ref::<i64>("key2").unwrap();
        assert_eq!(*value, 2);

        let value_1 = values.get_value("key").unwrap();
        let mut values = values.next();
        values.set("key2".into(), ValueRef::Borrowed(value_1));
        assert_eq!(*values.get_ref::<i64>("key2").unwrap(), 1);
    }
}
