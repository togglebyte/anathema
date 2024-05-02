use std::ops::Deref;

use anathema_state::CommonVal;
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
pub struct Attributes<'bp>(pub(crate) Values<'bp>);

impl<'bp> Attributes<'bp> {
    /// Get the `Value` out of attributes.
    /// This is always the first item
    pub fn value(&self) -> Option<&Value<'_, EvalValue<'_>>> {
        self.0.get(ValueIndex::ZERO)
    }

    // TODO this is get_copy, rename this
    pub fn get_c<T: 'static + Copy + PartialEq + TryFrom<CommonVal<'bp>>>(&self, key: &str) -> Option<T> {
        let value = self.get(key)?;
        value.load::<T>()
    }

    // TODO this is get_reference, rename this
    pub fn get_r<'a, T: TryFrom<&'a EvalValue<'bp>>>(&'a self, key: &str) -> Option<T> {
        self.get(key).and_then(|s| T::try_from(s.deref()).ok())
    }

    pub fn get(&self, key: &str) -> Option<&Value<'_, EvalValue<'bp>>> {
        let index = self.0.get_index(&ValueKey::Attribute(key))?;
        self.0.get(index)
    }

    pub fn get_index(&self, key: &str) -> Option<SmallIndex> {
        self.0.get_index(&ValueKey::Attribute(key))
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key).map(|val| val.load_bool()).unwrap_or(false)
    }

    pub(crate) fn get_mut(&mut self, index: ValueIndex) -> Option<&mut Value<'bp, EvalValue<'bp>>> {
        self.0.get_mut(index)
    }

    /// Iterate over attributes, skipping the first one
    /// as that is the `Value`.
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey<'_>, &Value<'_, EvalValue<'_>>)> {
        self.0.iter().skip(1)
    }
}
