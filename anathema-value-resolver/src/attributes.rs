use std::borrow::Borrow;

use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::SmallIndex;

use crate::expression::ValueExpr;
use crate::value::{Value, Values};
use crate::ValueKind;

type WidgetId = anathema_store::slab::Key;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum ValueKey<'bp> {
    #[default]
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
    pub(crate) attribs: Values<'bp>,
    pub value: Option<SmallIndex>,
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
            attribs: Values::empty(),
            value: None,
        }
    }

    pub fn set(&mut self, key: &'bp str, value: impl Into<ValueKind<'bp>>) {
        let key = ValueKey::Attribute(key);
        let value = value.into();
        let value = Value {
            expr: ValueExpr::Null,
            kind: value,
            sub: anathema_state::Subscriber::MAX,
            sub_to: anathema_state::SubTo::Zero,
        };

        self.attribs.set(key, value);
    }

    pub fn insert_with<F>(&mut self, key: ValueKey<'bp>, f: F) -> SmallIndex
    where
        F: FnMut(SmallIndex) -> Value<'bp>,
    {
        self.attribs.insert_with(key, f)
    }

    pub fn remove(&mut self, key: &'bp str) -> Option<Value<'bp>> {
        let key = ValueKey::Attribute(key);
        self.attribs.remove(&key)
    }

    /// Get the `Value` out of attributes.
    /// This is always the first item
    pub fn value(&self) -> Option<&ValueKind<'bp>> {
        let idx = self.value?;
        self.attribs.get_with_index(idx).map(|val| &val.kind)
    }

    pub fn get(&self, key: &str) -> Option<&ValueKind<'bp>> {
        self.attribs.get(key).map(|val| &val.kind)
    }

    pub fn get_as<'a, T>(&'a self, key: &str) -> Option<T>
    where
        T: TryFrom<&'a ValueKind<'bp>>,
    {
        self.attribs.get(key).and_then(|val| (&val.kind).try_into().ok())
    }

    pub fn iter_as<'a, T>(&'a self, key: &str) -> impl Iterator<Item = T>
    where
        T: TryFrom<&'a ValueKind<'bp>>,
    {
        self.attribs
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

    pub fn get_mut_with_index(&mut self, index: SmallIndex) -> Option<&mut Value<'bp>> {
        self.attribs.get_mut_with_index(index)
    }

    /// Iterate over attributes.
    /// This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &ValueKind<'bp>)> {
        self.attribs.iter().filter_map(|(key, val)| match key {
            ValueKey::Value => None,
            ValueKey::Attribute(_) => Some((key, &val.kind)),
        })
    }

    pub(super) fn get_value_expr(&self, key: &str) -> Option<ValueExpr<'bp>> {
        let value = self.attribs.get(key)?;
        Some(value.expr.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_as_int() {
        let mut attributes = Attributes::empty();
        let values = ValueKind::List([ValueKind::Int(1), ValueKind::Bool(true), ValueKind::Int(2)].into());
        attributes.set("a", values);

        let values = attributes.iter_as::<u8>("a").collect::<Vec<_>>();
        assert_eq!(vec![1, 2], values);

        let values = attributes.iter_as::<bool>("a").collect::<Vec<_>>();
        assert_eq!(vec![true], values);
    }
}
