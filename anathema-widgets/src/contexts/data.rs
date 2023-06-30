use std::collections::HashMap;

use crate::Value;

#[derive(Debug, Default)]
pub struct DataCtx(pub HashMap<String, Value>);

impl DataCtx {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn by_key(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn get_mut_value(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.0.get_mut(key)?.try_into().ok()
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.0.get(key)?.try_into().ok()
    }
}
