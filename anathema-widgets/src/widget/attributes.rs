use std::ops::Deref;

use anathema_state::{CommonVal, PendingValue};
use anathema_store::slab::{Gen, SecondaryMap};
use anathema_store::smallmap::SmallIndex;

use crate::expressions::EvalValue;
use crate::paint::CellAttributes;
use crate::values::Values;
use crate::widget::ValueKey;
use crate::{Value, WidgetId};

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

    /// Get a mutable reference to attributes by widget id
    pub fn get_mut(&mut self, id: WidgetId) -> &mut Attributes<'bp> {
        self.0
            .get_mut(id)
            .map(|(_, a)| a)
            .expect("every element has attributes")
    }

    /// Insert attributes for a given widget.
    ///
    /// This will overwrite any existing attributes at that location
    pub fn insert(&mut self, widget_id: WidgetId, attribs: Attributes<'bp>) {
        self.0.insert(widget_id, (widget_id.gen(), attribs))
    }

    /// Try to remove attributes for a specific widget
    pub fn try_remove(&mut self, id: WidgetId) {
        let _ = self.0.remove_if(id, |(current_gen, _)| *current_gen == id.gen());
    }
}

#[derive(Debug)]
pub struct Attributes<'bp> {
    pub(crate) values: Values<'bp>,
    pub(crate) value: Option<SmallIndex>,
    widget_id: WidgetId,
}

impl<'bp> Attributes<'bp> {
    /// Create an empty set of attributes
    pub fn empty(widget_id: WidgetId) -> Self {
        Self {
            values: Values::empty(),
            value: None,
            widget_id,
        }
    }

    /// Set the value
    pub fn set(&mut self, key: &'bp str, value: impl Into<CommonVal<'bp>>) {
        let value = value.into().into();
        self.values.set(ValueKey::Attribute(key), value);
    }

    /// Resolve the value from a state and track it from the attributes.
    /// This means changes to the state value will update the attribute automatically
    pub fn set_pending(&mut self, key: &'bp str, value: PendingValue) {
        let key = ValueKey::Attribute(key);
        match self.values.get_index(&key) {
            Some(idx) => {
                let valueref = value.to_value((self.widget_id, idx).into());
                self.values.set(key, valueref.into());
            }
            None => {
                self.values.insert_with(key, |idx| {
                    let valueref = value.to_value((self.widget_id, idx).into());
                    valueref.into()
                });
            }
        }
    }

    pub(crate) fn insert_with<F>(&mut self, key: ValueKey<'bp>, f: F) -> SmallIndex
    where
        F: Fn(SmallIndex) -> Value<'bp, EvalValue<'bp>>,
    {
        self.values.insert_with(key, f)
    }

    pub fn remove(&mut self, key: &'bp str) -> Option<Value<'_, EvalValue<'_>>> {
        let key = ValueKey::Attribute(key);
        self.values.remove(&key)
    }

    /// Get the `Value` out of attributes.
    /// This is always the first item
    pub fn value(&self) -> Option<&Value<'_, EvalValue<'_>>> {
        let idx = self.value?;
        self.values.get_with_index(idx)
    }

    /// Get a copy of a value
    pub fn get<T>(&self, key: &'bp str) -> Option<T>
    where
        T: 'static,
        T: Copy + PartialEq,
        for<'a> T: TryFrom<CommonVal<'a>>,
    {
        let value = self.get_val(key)?;
        value.load::<T>()
    }

    /// Get a reference to value
    /// ```
    /// # use anathema_widgets::{Attributes, WidgetId};
    /// let mut attributes = Attributes::empty(WidgetId::ZERO);
    /// let s = String::from("hello");
    /// attributes.set("str", s.as_ref());
    /// attributes.set("num", 123u32);
    /// assert_eq!("hello", attributes.get_ref::<&str>("str").unwrap());
    /// assert_eq!(123, attributes.get::<u32>("num").unwrap());
    /// ```
    pub fn get_ref<'a, T: TryFrom<&'a EvalValue<'bp>>>(&'a self, key: &'bp str) -> Option<T> {
        self.get_val(key).and_then(|s| T::try_from(s.deref()).ok())
    }

    pub fn get_val(&self, key: &'bp str) -> Option<&Value<'bp, EvalValue<'bp>>> {
        let key = ValueKey::Attribute(key);
        self.values.get(&key)
    }

    /// Get an integer regardless of how the value was stored.
    /// This will convert any state value of any numerical type
    /// into a `i64`.
    pub fn get_int(&self, key: &'bp str) -> Option<i64> {
        let key = ValueKey::Attribute(key);

        let value = self.values.get(&key)?;
        value
            .load_common_val()
            .and_then(|e| e.load_number().map(|n| n.as_int()))
    }

    /// Get an unsigned integer regardless of how the value was stored.
    /// This will convert any state value of any numerical type
    /// into a `usize`.
    /// This will truncate any bits don't fit into a usize.
    pub fn get_usize(&self, key: &'bp str) -> Option<usize> {
        let key = ValueKey::Attribute(key);

        let value = self.values.get(&key)?;
        value
            .load_common_val()
            .and_then(|e| e.load_number().map(|n| n.as_uint()))
    }

    pub(crate) fn get_mut_with_index(&mut self, index: SmallIndex) -> Option<&mut Value<'bp, EvalValue<'bp>>> {
        self.values.get_mut_with_index(index)
    }

    /// Treat the underlying value as a boolean.
    /// If it isn't it will default to false
    pub fn get_bool(&self, key: &'bp str) -> bool {
        self.get_val(key).map(|val| val.load_bool()).unwrap_or(false)
    }

    /// Iterate over attributes.
    /// This will skip the value
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &Value<'_, EvalValue<'_>>)> {
        self.values.iter().filter(|(key, _)| match key {
            ValueKey::Value => false,
            ValueKey::Attribute(_) => true,
        })
    }

    /// Returns true if the attributes contains the key
    pub fn contains(&self, key: &'bp str) -> bool {
        let key = ValueKey::Attribute(key);
        self.values.get(&key).is_some()
    }
}

impl CellAttributes for Attributes<'_> {
    fn with_str(&self, key: &str, f: &mut dyn FnMut(&str)) {
        let Some(value) = self.get_val(key).and_then(|value| value.load_common_val()) else { return };
        let Some(value) = value.to_common() else { return };
        let CommonVal::Str(s) = value else { return };
        f(s);
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_int(key)
    }

    fn get_u8(&self, key: &str) -> Option<u8> {
        self.get_int(key).map(|i| i as u8)
    }

    fn get_hex(&self, key: &str) -> Option<anathema_state::Hex> {
        let value = self.get_val(key)?;
        let value = value.load_common_val()?;
        match value.to_common()? {
            CommonVal::Hex(hex) => Some(hex),
            _ => None,
        }
    }

    fn get_color(&self, key: &str) -> Option<anathema_state::Color> {
        let value = self.get_val(key)?;
        let value = value.load_common_val()?;
        match value.to_common()? {
            CommonVal::Color(color) => Some(color),
            _ => None,
        }
    }

    fn get_bool(&self, key: &str) -> bool {
        Attributes::get_bool(self, key)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        let s = String::from("hello");
        attributes.set("str", s.as_ref());
        attributes.set("num", 123u32);

        assert_eq!("hello", attributes.get_ref::<&str>("str").unwrap());
        assert_eq!(123, attributes.get::<u32>("num").unwrap());
    }

    #[test]
    fn write_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        attributes.set("num", 1u32);
        assert_eq!(1, attributes.get::<u32>("num").unwrap());
    }

    #[test]
    fn remove_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        attributes.remove("num");
        assert!(attributes.get::<u32>("num").is_none());
    }

    #[test]
    fn contains_attribute() {
        let mut attributes = Attributes::empty(WidgetId::ZERO);
        attributes.set("num", 123u32);
        assert!(attributes.contains("num"));
    }
}
