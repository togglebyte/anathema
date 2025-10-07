use std::borrow::Borrow;

use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::ValueKind;
use crate::expression::ResolvedExpr;
use crate::value::Value;

type WidgetId = anathema_store::slab::Key;

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

// -----------------------------------------------------------------------------
//   - Attribute storage -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct AttributeStorage<'bp>(SecondaryMap<WidgetId, (Gen, Attributes<'bp>)>);

impl<'bp> AttributeStorage<'bp> {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    /// Get a reference to attributes by widget id
    pub fn get(&self, id: WidgetId) -> &Attributes<'bp> {
        self.0.get(id).map(|(_, a)| a).expect("every element has attributes")
    }

    /// Get a reference to attributes by widget id
    pub fn try_get(&self, id: WidgetId) -> Option<&Attributes<'bp>> {
        self.0.get(id).map(|(_, a)| a)
    }

    /// Get a mutable reference to attributes by widget id
    pub fn get_mut(&mut self, id: WidgetId) -> &mut Attributes<'bp> {
        self.0
            .get_mut(id)
            .map(|(_, a)| a)
            .expect("every element has attributes")
    }

    pub fn with_mut<F, O>(&mut self, widget_id: WidgetId, f: F) -> Option<O>
    where
        F: FnOnce(&mut Attributes<'bp>, &mut Self) -> O,
    {
        let mut value = self.try_remove(widget_id)?;
        let output = f(&mut value, self);
        self.insert(widget_id, value);
        Some(output)
    }

    /// Insert attributes for a given widget.
    ///
    /// This will overwrite any existing attributes at that location
    pub fn insert(&mut self, widget_id: WidgetId, attribs: Attributes<'bp>) {
        self.0.insert(widget_id, (widget_id.generation(), attribs))
    }

    /// Try to remove attributes for a specific widget
    pub fn try_remove(&mut self, id: WidgetId) -> Option<Attributes<'bp>> {
        self.0
            .remove_if(id, |(current_gen, _)| *current_gen == id.generation())
            .map(|(_, value)| value)
    }
}

// -----------------------------------------------------------------------------
//   - Attributes -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Attributes<'bp> {
    inner: SmallMap<ValueKey<'bp>, Value<'bp>>,
}

// TODO
// Only get, set and remove should be part of the interface
// that is exposed to the end user.
//
// The rest is for widget creation and should be moved to its own type.

impl<'bp> Attributes<'bp> {
    /// Create an empty set of attributes
    pub fn empty() -> Self {
        Self {
            inner: SmallMap::empty(),
        }
    }

    /// Set an attribute value.
    /// ```
    /// # use anathema_value_resolver::{Attributes, ValueKind};
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set("name", "Nonsense");
    /// attributes.get_as::<&str>("name").unwrap();
    /// ```
    pub fn set(&mut self, key: &'bp str, value: impl Into<ValueKind<'bp>>) {
        let key = ValueKey::Attribute(key);
        let value = Value::static_val(value);
        self.inner.set(key, value);
    }

    /// Set an attribute value.
    /// ```
    /// # use anathema_value_resolver::{Attributes, ValueKind};
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set_value("Nonsense");
    /// attributes.value_as::<&str>().unwrap();
    /// ```
    pub fn set_value(&mut self, value: impl Into<ValueKind<'bp>>) {
        let key = ValueKey::Value;
        let value = Value::static_val(value);
        self.inner.set(key, value);
    }

    // This is only used for inserting values during widget creation where values originate from
    // expressions.
    #[doc(hidden)]
    pub fn insert_with<F>(&mut self, key: ValueKey<'bp>, f: F) -> SmallIndex
    where
        F: FnMut(SmallIndex) -> Value<'bp>,
    {
        self.inner.insert_with(key, f)
    }

    /// Remove a value from attributes
    pub fn remove(&mut self, key: &'bp str) -> Option<Value<'bp>> {
        let key = ValueKey::Attribute(key);
        self.inner.remove(&key)
    }

    /// Get the `Value` out of attributes.
    /// This is always the first item
    pub fn value(&self) -> Option<&ValueKind<'bp>> {
        self.inner.get(&ValueKey::Value).map(|val| &val.kind)
    }

    /// Get a value as a specific type
    pub fn value_as<'a, T>(&'a self) -> Option<T>
    where
        T: TryFrom<&'a ValueKind<'bp>>,
    {
        self.inner
            .get(&ValueKey::Value)
            .and_then(|val| (&val.kind).try_into().ok())
    }

    pub fn get(&self, key: &str) -> Option<&ValueKind<'bp>> {
        self.inner.get(key).map(|val| &val.kind)
    }

    /// Get a value as a given type.
    /// If the value doesn't exist or can not be cast to the
    /// expected type `None` is returned.
    /// ```
    /// # use anathema_value_resolver::{Attributes, ValueKind};
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set("num", 123);
    ///
    /// assert_eq!(attributes.get_as::<u32>("num").unwrap(), 123);
    /// assert_eq!(attributes.get_as::<i16>("num").unwrap(), 123);
    /// ```
    pub fn get_as<'a, T>(&'a self, key: &str) -> Option<T>
    where
        T: TryFrom<&'a ValueKind<'bp>>,
    {
        self.inner.get(key).and_then(|val| (&val.kind).try_into().ok())
    }

    /// Iterate over values of a given type
    /// ```
    /// # use anathema_value_resolver::{Attributes, ValueKind};
    ///
    /// let mut attributes = Attributes::empty();
    /// let values =
    ///     ValueKind::List([ValueKind::Int(1), ValueKind::Bool(true), ValueKind::Int(2)].into());
    /// attributes.set("mixed_list", values);
    ///
    /// let iter = attributes.iter_as::<u32>("mixed_list");
    /// assert_eq!(vec![1u32, 2], iter.collect::<Vec<_>>());
    /// ```
    pub fn iter_as<'a, T>(&'a self, key: &str) -> impl Iterator<Item = T>
    where
        T: TryFrom<&'a ValueKind<'bp>>,
    {
        self.inner
            .get(key)
            .and_then(|val| match &val.kind {
                ValueKind::List(value_kinds) => {
                    let list = value_kinds.iter().filter_map(|v| T::try_from(v).ok());
                    Some(list)
                }

                _ => None,
            })
            .into_iter()
            .flatten()
    }

    /// Iterate over attributes.
    /// This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &ValueKind<'bp>)> {
        self.inner.iter().filter_map(|(key, val)| match key {
            ValueKey::Value => None,
            ValueKey::Attribute(_) => Some((key, &val.kind)),
        })
    }

    #[doc(hidden)]
    /// This should only be used internally by the widgets
    /// when updating a value.
    pub fn get_mut_with_index(&mut self, index: SmallIndex) -> Option<&mut Value<'bp>> {
        self.inner.get_mut_with_index(index)
    }

    pub(super) fn get_value_expr(&self, key: &str) -> Option<ResolvedExpr<'bp>> {
        let value = self.inner.get(key)?;
        Some(value.expr.clone())
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{Color, Hex};

    use super::*;

    fn attribs() -> Attributes<'static> {
        let mut attributes = Attributes::empty();

        let values = ValueKind::List([ValueKind::Int(1), ValueKind::Bool(true), ValueKind::Int(2)].into());
        attributes.set("mixed_list", values);
        attributes.set("num", 123);
        attributes.set("static_str", "static");
        attributes.set("string", String::from("string"));
        attributes.set("hex", Hex::from((1, 2, 3)));
        attributes.set("red", Color::Red);
        attributes.set("float", 1.23);
        attributes.set("bool", true);
        attributes.set("char", 'a');
        attributes.set_value(123u8);

        attributes
    }

    #[test]
    fn iter_as_type() {
        let attributes = attribs();

        let values = attributes.iter_as::<u8>("mixed_list").collect::<Vec<_>>();
        assert_eq!(vec![1, 2], values);

        let values = attributes.iter_as::<bool>("mixed_list").collect::<Vec<_>>();
        assert_eq!(vec![true], values);
    }

    #[test]
    fn get_as_int() {
        assert_eq!(123, attribs().get_as::<u8>("num").unwrap());
        assert_eq!(123, attribs().get_as::<i8>("num").unwrap());
        assert_eq!(123, attribs().get_as::<u16>("num").unwrap());
        assert_eq!(123, attribs().get_as::<i16>("num").unwrap());
        assert_eq!(123, attribs().get_as::<u32>("num").unwrap());
        assert_eq!(123, attribs().get_as::<i32>("num").unwrap());
        assert_eq!(123, attribs().get_as::<u64>("num").unwrap());
        assert_eq!(123, attribs().get_as::<i64>("num").unwrap());
        assert_eq!(123, attribs().get_as::<usize>("num").unwrap());
        assert_eq!(123, attribs().get_as::<isize>("num").unwrap());
    }

    #[test]
    fn get_as_strings() {
        let attributes = attribs();
        assert_eq!("static", attributes.get_as::<&str>("static_str").unwrap());
        assert_eq!("string", attributes.get_as::<&str>("string").unwrap());
    }

    #[test]
    fn get_as_hex() {
        let attributes = attribs();
        assert_eq!(Hex::from((1, 2, 3)), attributes.get_as::<Hex>("hex").unwrap());
    }

    #[test]
    fn get_as_color() {
        let attributes = attribs();
        assert_eq!(Color::Red, attributes.get_as::<Color>("red").unwrap());
    }

    #[test]
    fn get_as_float() {
        let attributes = attribs();
        assert_eq!(1.23, attributes.get_as::<f32>("float").unwrap());
        assert_eq!(1.23, attributes.get_as::<f64>("float").unwrap());
    }

    #[test]
    fn get_as_bool() {
        let attributes = attribs();
        assert!(attributes.get_as::<bool>("bool").unwrap());
    }

    #[test]
    fn get_as_char() {
        let attributes = attribs();
        assert_eq!('a', attributes.get_as::<char>("char").unwrap());
    }

    #[test]
    fn get_value_as() {
        let attributes = attribs();
        let value = attributes.value_as::<u8>().unwrap();
        assert_eq!(value, 123);
    }
}
