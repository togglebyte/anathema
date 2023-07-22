use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anathema_values::Value;

use crate::views::ViewCollection;

#[derive(Debug, Default)]
pub struct DataCtx {
    data: HashMap<String, Value>,
    pub views: ViewCollection,
}

impl DataCtx {
    pub(crate) fn by_key(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.data.insert(key.into(), value.into());
    }

    pub fn get_mut_or<T: 'static>(&mut self, key: &str, or_val: T) -> &mut T
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
        Value: From<T>,
    {
        match self.data.entry(key.into()) {
            Entry::Vacant(e) => e.insert(or_val.into()),
            Entry::Occupied(e) => e.into_mut(),
        }
        .try_into()
        .ok()
        .expect("this can't fail as we assure that the value exist")
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.data.get_mut(key)?.try_into().ok()
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.data.get(key)?.try_into().ok()
    }
}
