use std::collections::HashMap;

use anathema_values::Value;

// -----------------------------------------------------------------------------
//     - Attributes -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Attributes {
    pub(crate) inner: HashMap<String, Value>,
}

impl std::ops::Index<&str> for Attributes {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        &self.inner[index]
    }
}

impl From<Vec<Attribute<'_>>> for Attributes {
    fn from(attributes: Vec<Attribute<'_>>) -> Self {
        let mut inner = HashMap::new();

        for attr in attributes {
            inner.insert(attr.key.to_owned(), attr.val);
        }

        Self { inner }
    }
}

impl From<()> for Attributes {
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

impl<const N: usize, K: Into<String>, T: Into<Value>> From<[(K, T); N]> for Attributes {
    fn from(src: [(K, T); N]) -> Self {
        let mut attributes = Self::empty();
        for (k, v) in src {
            attributes.set(k, v);
        }

        attributes
    }
}

impl Attributes {
    pub fn empty() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn new(key: &str, value: impl Into<Value>) -> Self {
        let mut inner = Self::empty();
        inner.set(key, value.into());
        inner
    }

    pub fn has(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.inner.get(name)
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        self.inner.insert(name.into(), value.into());
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type IntoIter = std::collections::hash_map::Iter<'a, String, Value>;
    type Item = (&'a String, &'a Value);

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Attributes {
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Value>;
    type Item = (&'a String, &'a mut Value);

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

// -----------------------------------------------------------------------------
//     - Attribute -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Attribute<'src> {
    pub key: &'src str,
    pub val: Value,
}

#[cfg(test)]
mod test {

    // // -----------------------------------------------------------------------------
    // //     - prop tests -
    // // -----------------------------------------------------------------------------
    // proptest! {
    //     #[test]
    //     fn parse_random_string_attribs(attrib in any::<String>()) {
    //         let attrib = attrib.replace('"', "");
    //         let attrib = attrib.replace('\\', "");
    //         let mut attributes = attribs(&format!("container [attrib:\"{attrib}\"]:"));
    //         let actual = attributes.get_value("attrib");
    //         let expected = Some(Value::String(attrib));
    //         assert_eq!(expected, actual);
    //     }
    // }
}
