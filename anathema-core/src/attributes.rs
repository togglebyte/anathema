//! Element attributes
use std::borrow::Borrow;
// #![deny(missing_docs)]
use std::ops::Deref;

use anathema_store::remotecell::RemoteCell;
use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::runtime::elements::ElementId;
use crate::runtime::eval::values::TemplateValue;

#[derive(Debug)]
pub struct AllAttributes<'bp> {
    attributes: SecondaryMap<ElementId, Attributes<'bp>>,
}

impl<'bp> AllAttributes<'bp> {
    pub(crate) fn empty() -> Self {
        Self {
            attributes: SecondaryMap::empty(),
        }
    }

    pub(crate) fn insert(&mut self, id: ElementId, attributes: Attributes<'bp>) {
        self.attributes.insert(id, attributes);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueKey<'bp> {
    Value,
    Attribute(&'bp str),
}

impl ValueKey<'_> {
    pub fn as_str(&self) -> &str {
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

#[derive(Debug)]
pub struct Attributes<'bp> {
    inner: SmallMap<ValueKey<'bp>, RemoteCell<TemplateValue<'bp>>>,
}

impl<'bp> Attributes<'bp> {
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

    pub(crate) fn get(&self, key: &str) -> Option<&RemoteCell<TemplateValue<'bp>>> {
        self.inner.get(key)
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

    /// Iterate over attributes.
    /// This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &RemoteCell<TemplateValue<'bp>>)> {
        self.inner.iter().filter_map(|(key, val)| match key {
            ValueKey::Value => None,
            ValueKey::Attribute(_) => Some((key, &*val)),
        })
    }
}
