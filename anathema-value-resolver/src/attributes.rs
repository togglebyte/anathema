use std::borrow::Borrow;

use anathema_state::PendingValue;
use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::SmallIndex;

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
        self.0.insert(widget_id, (widget_id.gen(), attribs))
    }

    /// Try to remove attributes for a specific widget
    pub fn try_remove(&mut self, id: WidgetId) -> Option<Attributes<'bp>> {
        self.0
            .remove_if(id, |(current_gen, _)| *current_gen == id.gen())
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
    widget_id: WidgetId,
}

impl<'bp> Attributes<'bp> {
    /// Create an empty set of attributes
    pub fn empty(widget_id: WidgetId) -> Self {
        Self {
            attribs: Values::empty(),
            value: None,
            widget_id,
        }
    }

    /// Set the value. This should only be used when evaluating new widgets,
    /// and should not be used by user code.
    pub fn set(&mut self, key: &'bp str, value: Value<'bp>) {
        self.attribs.set(ValueKey::Attribute(key), value);
    }

    /// Resolve the value from a state and track it from the attributes.
    /// This means changes to the state value will update the attribute automatically
    pub fn set_pending(&mut self, key: &'bp str, value: PendingValue) {
        let key = ValueKey::Attribute(key);
        match self.attribs.get_index(&key) {
            Some(idx) => {
                let valueref = value.subscribe((self.widget_id, idx).into());
                panic!();
                // self.attribs.set(key, valueref.into());
            }
            None => {
                self.attribs.insert_with(key, |idx| {
                    let valueref = value.subscribe((self.widget_id, idx).into());
                    panic!()
                    // valueref.into()
                });
            }
        }
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

    // TODO: is this ever used?
    // /// Get a copy of a value
    // pub fn get<T>(&self, key: &'bp str) -> Option<T>
    // where
    //     T: 'static,
    //     T: Copy + PartialEq,
    //     T: TryFrom<Value<'bp>>,
    // {
    //     let value = self.get_val(key)?;
    //     value.load::<T>()
    // }

    // TODO: is this ever used?
    // /// Get a reference to value
    // /// ```
    // /// # use anathema_widgets::{Attributes, WidgetId};
    // /// let mut attributes = Attributes::empty(WidgetId::ZERO);
    // /// let s = String::from("hello");
    // /// attributes.set("str", s.as_ref());
    // /// attributes.set("num", 123u32);
    // /// assert_eq!("hello", attributes.get_ref::<&str>("str").unwrap());
    // /// assert_eq!(123, attributes.get::<u32>("num").unwrap());
    // /// ```
    // pub fn get_ref<'a, T: TryFrom<&'a Value<'bp>>>(&'a self, key: &'bp str) -> Option<T> {
    //     self.get_val(key).and_then(|s| T::try_from(s.deref()).ok())
    // }

    pub fn get(&self, key: &str) -> Option<&ValueKind<'bp>> {
        self.attribs.get(key).map(|val| &val.kind)
    }

    pub fn get_as<T>(&self, key: &str) -> Option<T>
    where
        T: for<'a> TryFrom<&'a ValueKind<'a>>,
    {
        self.attribs.get(key).and_then(|val| (&val.kind).try_into().ok())
    }

    /// Get an integer regardless of how the value was stored.
    /// This will convert any state value of any numerical type
    /// into a `i64`.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        let key = ValueKey::Attribute(key);
        let value = self.attribs.get(&key)?;
        value.as_int()
    }

    /// Get an unsigned integer regardless of how the value was stored.
    /// This will convert any state value of any numerical type
    /// into a `usize`.
    /// This will truncate any bits don't fit into a usize.
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.get_int(key).map(|val| val as usize)
    }

    pub fn get_mut_with_index(&mut self, index: SmallIndex) -> Option<&mut Value<'bp>> {
        self.attribs.get_mut_with_index(index)
    }

    /// Treat the underlying value as a boolean.
    /// If it isn't it will default to false
    pub fn get_bool(&self, key: &'bp str) -> bool {
        let key = ValueKey::Attribute(key);
        let Some(value) = self.attribs.get(&key) else { return false };
        value.as_bool().unwrap_or(false)
    }

    /// Iterate over attributes.
    /// This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &ValueKind<'bp>)> {
        self.attribs.iter().filter_map(|(key, val)| match key {
            ValueKey::Value => None,
            ValueKey::Attribute(_) => Some((key, &val.kind)),
        })
    }

    /// Returns true if the attributes contains the key
    pub fn contains(&self, key: &'bp str) -> bool {
        let key = ValueKey::Attribute(key);
        self.attribs.get(&key).is_some()
    }
}
