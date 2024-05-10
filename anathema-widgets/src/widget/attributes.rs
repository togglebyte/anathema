use std::ops::Deref;

use anathema_state::{CommonVal, PendingValue};
use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::SmallIndex;

use crate::expressions::EvalValue;
use crate::values::{ValueIndex, Values};
use crate::widget::ValueKey;
use crate::{Value, WidgetId};

pub struct AttributeStorage<'bp>(SecondaryMap<Attributes<'bp>>);

impl<'bp> AttributeStorage<'bp> {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn get(&self, id: WidgetId) -> &Attributes<'bp> {
        self.0.get(id).expect("every element has attributes")
    }

    pub fn get_mut(&mut self, id: WidgetId) -> &mut Attributes<'bp> {
        self.0.get_mut(id).expect("every element has attributes")
    }

    pub fn insert(&mut self, widget_id: WidgetId, attribs: Attributes<'bp>) {
        self.0.insert(widget_id, attribs)
    }

    pub fn remove(&mut self, id: WidgetId) {
        self.0.remove(id);
    }
}

// TODO
// At the time of writing the attributes were read-only.
// The inner values could change, but the attributes them selves
// would never be removed or new ones inserted.
//
// After some consideration this turned out to be a bad idea.
// Therefore we don't need the index anymore and the entire underlying storage
// should be replaced with something that can have new values added, old values removed.
#[derive(Debug)]
pub struct Attributes<'bp> {
    pub(crate) values: Values<'bp>,
    widget_id: WidgetId,
}

impl<'bp> Attributes<'bp> {
    /// Create an empty set of attributes
    pub fn empty(widget_id: WidgetId) -> Self {
        Self {
            values: Values::empty(),
            widget_id,
        }
    }

    /// Set the value
    pub fn set(&mut self, key: &'bp str, value: impl Into<CommonVal<'bp>>) {
        let value = value.into();
        self.values.set(ValueKey::Attribute(key), value.into());
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

    pub(crate) fn insert_with<F>(&mut self, key: ValueKey<'bp>, f: F)
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
        self.values.get_with_index(ValueIndex::ZERO)
    }

    pub fn get<T>(&self, key: &'bp str) -> Option<T>
    where
        T: 'static,
        T: Copy + PartialEq,
        for<'a> T: TryFrom<CommonVal<'a>>,
    {
        let value = self.get_val(key)?;
        value.load::<T>()
    }

    pub fn get_ref<'a, T: TryFrom<&'a EvalValue<'bp>>>(&'a self, key: &'bp str) -> Option<T> {
        self.get_val(key).and_then(|s| T::try_from(s.deref()).ok())
    }

    pub fn get_val(&self, key: &'bp str) -> Option<&Value<'_, EvalValue<'bp>>> {
        let key = ValueKey::Attribute(key);
        self.values.get(&key)
    }

    pub(crate) fn get_mut_with_index(&mut self, index: SmallIndex) -> Option<&mut Value<'bp, EvalValue<'bp>>> {
        self.values.get_mut_with_index(index)
    }

    pub fn get_bool(&self, key: &'bp str) -> bool {
        self.get_val(key).map(|val| val.load_bool()).unwrap_or(false)
    }

    /// Iterate over attributes, skipping the first one
    /// as that is the `Value`.
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &Value<'_, EvalValue<'_>>)> {
        self.values.iter().skip(1)
    }

    pub(crate) fn contains(&self, key: &ValueKey<'bp>) -> bool {
        self.values.get(key).is_some()
    }
}
