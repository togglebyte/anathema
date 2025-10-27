//! Element attributes
use std::borrow::Borrow;
use std::ops::Deref;

use anathema_store::remotecell::RemoteCell;
use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::runtime::elements::ElementId;
use crate::runtime::eval::values::TemplateValue;

// All attributes for all elements
#[derive(Debug)]
pub(crate) struct AttributeRegistry<'bp> {
    attributes: SecondaryMap<ElementId, Attributes<'bp>>,
}

impl<'bp> AttributeRegistry<'bp> {
    pub(crate) fn empty() -> Self {
        Self {
            attributes: SecondaryMap::empty(),
        }
    }

    pub(crate) fn insert(&mut self, id: ElementId, attributes: Attributes<'bp>) {
        self.attributes.insert(id, attributes);
    }

    pub(crate) fn get(&self, id: ElementId) -> Option<&Attributes<'bp>> {
        self.attributes.get(id)
    }
}

// The access key for attributes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum ValueKey<'bp> {
    Value,
    Attribute(&'bp str),
}

impl ValueKey<'_> {
    fn as_str(&self) -> &str {
        match self {
            ValueKey::Value => "[value]",
            ValueKey::Attribute(name) => name,
        }
    }
}

impl Borrow<str> for ValueKey<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// Element attributes
///
/// ```text
/// text [foreground: "red", bold: true] "hello world"
/// ```
///
/// Attributes can be set / replaced or read, but never mutated because of the `RemoteCell`.
#[derive(Debug)]
pub struct Attributes<'bp> {
    inner: SmallMap<ValueKey<'bp>, RemoteCell<TemplateValue<'bp>>>,
}

impl<'bp> Attributes<'bp> {
    /// Create a new instance of en empty set of attributes
    pub fn empty() -> Self {
        Self {
            inner: SmallMap::empty(),
        }
    }

    /// Set an attribute value.
    /// ```
    /// # use anathema_core::attributes::Attributes;
    /// # use anathema_core::runtime::TemplateValue;
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set("name", "Nonsense");
    /// attributes.get_as::<&str>("name").unwrap();
    /// ```
    pub fn set(&mut self, key: &'bp str, value: impl Into<TemplateValue<'bp>>) {
        let value = value.into();
        let (cell, _) = RemoteCell::new(value);
        self.inner.set(ValueKey::Attribute(key), cell);
    }

    /// Set an attribute value.
    /// ```
    /// # use anathema_core::attributes::Attributes ;
    /// # use anathema_core::runtime::TemplateValue;
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set_value("Nonsense");
    /// attributes.value_as::<&str>().unwrap();
    /// ```
    pub fn set_value(&mut self, value: impl Into<TemplateValue<'bp>>) {
        let key = ValueKey::Value;
        let value = value.into();
        let (cell, _) = RemoteCell::new(value);
        self.inner.set(key, cell);
    }

    // Set remote cells directly.
    // Use this when there is an update handle at the other end
    pub(crate) fn set_attribute(&mut self, key: ValueKey<'bp>, value: RemoteCell<TemplateValue<'bp>>) {
        self.inner.set(key, value);
    }

    /// Remove a value from attributes
    pub fn remove(&mut self, key: &str) -> Option<RemoteCell<TemplateValue<'bp>>> {
        self.inner.remove(key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&TemplateValue<'bp>> {
        let val = self.inner.get(key)?;
        Some(&*val)
    }

    /// Get a value as a given type.
    /// If the value doesn't exist or can not be cast to the
    /// expected type `None` is returned.
    /// ```
    /// # use anathema_core::attributes::Attributes;
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set("num", 123);
    ///
    /// assert_eq!(attributes.get_as::<u32>("num").unwrap(), 123);
    /// assert_eq!(attributes.get_as::<i16>("num").unwrap(), 123);
    /// ```
    pub fn get_as<'a, T>(&'a self, key: &str) -> Option<T>
    where
        T: TryFrom<&'a TemplateValue<'bp>>,
    {
        self.inner.get(key).and_then(|val| val.deref().try_into().ok())
    }

    /// Get a value as a specific type
    pub fn value_as<'a, T>(&'a self) -> Option<T>
    where
        T: TryFrom<&'a TemplateValue<'bp>>,
        T: ?Sized,
    {
        self.inner
            .get(&ValueKey::Value)
            .and_then(|val| val.deref().try_into().ok())
    }

    /// Get the `Value` out of attributes.
    /// This is always the first item
    pub fn value(&self) -> Option<&RemoteCell<TemplateValue<'bp>>> {
        self.inner.get(&ValueKey::Value)
    }

    /// Iterate over values of a given type
    /// ```
    /// # use anathema_core::attributes::Attributes;
    /// # use anathema_core::runtime::TemplateValue;
    ///
    /// let mut attributes = Attributes::empty();
    /// let values = TemplateValue::List(
    ///     [
    ///         TemplateValue::Int(1),
    ///         TemplateValue::Bool(true),
    ///         TemplateValue::Int(2),
    ///     ]
    ///     .into(),
    /// );
    /// attributes.set("mixed_list", values);
    ///
    /// let iter = attributes.iter_as::<u32>("mixed_list");
    /// assert_eq!(vec![1u32, 2], iter.collect::<Vec<_>>());
    /// ```
    pub fn iter_as<'a, T>(&'a self, key: &str) -> impl Iterator<Item = T>
    where
        T: TryFrom<&'a TemplateValue<'bp>>,
    {
        self.inner
            .get(key)
            .and_then(|val| match &**val {
                TemplateValue::List(list) => {
                    let list = list.iter().filter_map(|v| T::try_from(v).ok());
                    Some(list)
                }
                _ => None,
            })
            .into_iter()
            .flatten()
    }

    /// Iterator of keys and values.
    /// NOTE: This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TemplateValue<'bp>)> {
        self.inner.iter().filter_map(|(key, val)| match key {
            ValueKey::Value => None,
            &ValueKey::Attribute(key) => Some((key, &**val)),
        })
    }

    /// Iterator of keys
    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().filter_map(|(key, _)| match key {
            ValueKey::Value => None,
            &ValueKey::Attribute(key) => Some(key),
        })
    }

    /// Iterator of values
    pub fn iter_values(&self) -> impl Iterator<Item = &TemplateValue<'bp>> {
        self.inner.iter().filter_map(|(key, value)| match key {
            ValueKey::Value => None,
            &ValueKey::Attribute(_) => Some(&**value),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn attributes() -> Attributes<'static> {
        let mut attributes = Attributes::empty();
        attributes.set("int", 1);
        attributes.set("strings", vec!["one", "two"]);
        attributes.set("numbers", vec![1, 2, 3]);
        attributes.set(
            "mixed",
            TemplateValue::List(vec![true.into(), 1.into(), "string".into()].into()),
        );
        attributes
    }

    #[test]
    fn replace_attribute() {
        let mut attributes = attributes();
        assert_eq!(attributes.get_as::<u32>("int").unwrap(), 1);
        attributes.set("int", 2);
        assert_eq!(attributes.get_as::<u32>("int").unwrap(), 2);
    }

    #[test]
    fn set_value() {
        let mut attributes = attributes();
        attributes.set("other", 2);
        assert_eq!(attributes.get_as::<u32>("other").unwrap(), 2);
    }

    #[test]
    fn remove() {
        let mut attributes = attributes();
        attributes.remove("int");
        assert!(attributes.get("int").is_none());
    }

    #[test]
    fn get_attribute() {
        let attributes = attributes();
        assert_eq!(attributes.get("int"), Some(&TemplateValue::Int(1)));
    }

    #[test]
    fn get_attribute_as() {
        let attributes = attributes();
        assert_eq!(attributes.get_as::<u8>("int").unwrap(), 1);
        assert!(attributes.get_as::<bool>("int").is_none());
    }

    #[test]
    fn get_value_as() {
        let mut attributes = attributes();
        attributes.set_value("hello world");
        assert_eq!(attributes.value_as::<&str>().unwrap(), "hello world");

        attributes.set_value("hello world".to_string());
        assert_eq!(attributes.value_as::<&str>().unwrap(), "hello world");
    }

    #[test]
    fn iterate_as() {
        let attributes = attributes();
        let numbers = attributes.iter_as::<u32>("numbers").collect::<Vec<_>>();
        assert_eq!(numbers, vec![1, 2, 3]);

        let strings = attributes.iter_as::<&str>("strings").collect::<Vec<_>>();
        assert_eq!(strings, vec!["one", "two"]);
    }

    #[test]
    fn iter() {
        let attributes = attributes();
        let mut iter = attributes.iter();
        assert_eq!(("int", &TemplateValue::Int(1)), iter.next().unwrap());
    }

    #[test]
    fn iter_keys() {
        let attributes = attributes();
        let mut iter = attributes.iter_keys();
        assert_eq!("int", iter.next().unwrap());
        assert_eq!("strings", iter.next().unwrap());
        assert_eq!("numbers", iter.next().unwrap());
    }

    #[test]
    fn mixed_type_iteration() {
        let attributes = attributes();
        let mixed = attributes.get("mixed").unwrap();

        if let TemplateValue::List(list) = mixed {
            assert_eq!(list[0].as_bool().unwrap(), true);
            assert_eq!(list[1].as_int().unwrap(), 1);
            assert_eq!(list[2].as_str().unwrap(), "string");
        } else {
            panic!("Expected List");
        }
    }
}
