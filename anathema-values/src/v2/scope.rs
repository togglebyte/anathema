use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use crate::hashmap::HashMap;
use crate::{NodeId, Path, State};

#[derive(Debug)]
pub enum Collection {
    Rc(Rc<[ScopeValue]>),
    State { path: Path, len: usize },
    Empty,
}

impl Collection {
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Rc(col) => col.len(),
            Self::State { len, .. } => *len,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScopeValue {
    Static(Rc<str>),
    List(Rc<[ScopeValue]>),
    Dyn(Path),
}

impl<const N: usize> From<[ScopeValue; N]> for ScopeValue {
    fn from(arr: [ScopeValue; N]) -> Self {
        if N == 1 {
            arr.into_iter()
                .next()
                .expect("this is always going to be an array with a size of one")
        } else {
            ScopeValue::List(Rc::new(arr))
        }
    }
}

// TODO: add a testing flag for this
impl From<i32> for ScopeValue {
    fn from(s: i32) -> Self {
        Self::Static(s.to_string().into())
    }
}

// TODO: add a testing flag for this
impl From<String> for ScopeValue {
    fn from(s: String) -> Self {
        Self::Static(s.into())
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    inner: HashMap<Path, Cow<'a, ScopeValue>>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            inner: HashMap::new(),
            parent,
        }
    }

    pub fn scope(&mut self, path: Path, value: Cow<'a, ScopeValue>) {
        self.inner.insert(path, value);
    }

    /// Scope a value for a collection.
    /// TODO: Review if the whole cloning business here makes sense
    pub fn scope_collection(&mut self, binding: Path, collection: &Collection, value_index: usize) {
        let value = match collection {
            Collection::Rc(list) => Cow::Owned(list[value_index].clone()),
            Collection::State { path, .. } => {
                let path = path.compose(value_index);
                Cow::Owned(ScopeValue::Dyn(path))
            }
            Collection::Empty => return,
        };

        self.scope(binding, value);
    }

    pub fn lookup_parent(&self, path: &Path) -> Option<&'a ScopeValue> {
        self.parent.and_then(|parent| parent.lookup(path))
        // .map(Deref::deref)
    }

    pub fn lookup(&self, path: &Path) -> Option<&ScopeValue> {
        self.inner
            .get(path)
            .map(Deref::deref)
            .or_else(|| self.parent.and_then(|parent| parent.lookup(path)))
    }

    pub fn lookup_list(&self, path: &Path) -> Option<Rc<[ScopeValue]>> {
        let value = self
            .inner
            .get(path)
            .map(Deref::deref)
            .or_else(|| self.parent.and_then(|parent| parent.lookup(path)));

        match value {
            Some(ScopeValue::List(value)) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn from_self(&'a self) -> Scope<'a> {
        Scope::new(Some(self))
    }
}

pub struct Context<'a, 'val> {
    pub state: &'a mut dyn State,
    pub scope: &'a mut Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a mut dyn State, scope: &'a mut Scope<'val>) -> Self {
        Self { state, scope }
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<T>(&self, path: &Path, node_id: &NodeId) -> Option<T>
    where
        T: for<'magic> TryFrom<&'magic str>,
    {
        match self.scope.lookup(&path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get(path, node_id),
                ScopeValue::Static(s) => T::try_from(s).ok(),
                ScopeValue::List(_) => None,
            },
            None => self
                .state
                .get(&path, node_id)
                .and_then(|val| val.as_ref().try_into().ok()),
        }
    }

    pub fn attribute<T>(
        &self,
        key: impl AsRef<str>,
        node_id: &NodeId,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<T>
    where
        T: for<'magic> TryFrom<&'magic str>,
    {
        let attrib = attributes.get(key.as_ref())?;

        match attrib {
            ScopeValue::Static(val) => val.as_ref().try_into().ok(),
            ScopeValue::Dyn(path) => self.get(path, node_id),
            _ => None,
        }
    }

    pub fn primitive<T>(
        &self,
        key: impl AsRef<str>,
        node_id: &NodeId,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<T>
    where
        T: FromStr,
    {
        let attrib = attributes.get(key.as_ref())?;

        match attrib {
            ScopeValue::Static(val) => T::from_str(val.as_ref()).ok(),
            ScopeValue::Dyn(path) => self
                .get::<String>(path, node_id)
                .as_deref()
                .and_then(|s| T::from_str(s).ok()),
            _ => None,
        }
    }

    pub fn list_to_string(&self, list: &Rc<[ScopeValue]>, buffer: &mut String, node_id: &NodeId) {
        for val in list.iter() {
            match val {
                ScopeValue::List(list) => self.list_to_string(list, buffer, node_id),
                ScopeValue::Dyn(path) => buffer.push_str(&self.get_string(path, node_id)),
                ScopeValue::Static(s) => buffer.push_str(s),
            }
        }
    }

    pub fn list_to_string_no_sub(&self, list: &Rc<[ScopeValue]>, buffer: &mut String) {
        for val in list.iter() {
            match val {
                ScopeValue::List(list) => self.list_to_string_no_sub(list, buffer),
                ScopeValue::Dyn(path) => buffer.push_str(&self.get_string_no_sub(path)),
                ScopeValue::Static(s) => buffer.push_str(s),
            }
        }
    }

    pub fn get_string(&self, path: &Path, node_id: &NodeId) -> String {
        match self.scope.lookup(path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get_string(path, node_id),
                ScopeValue::Static(s) => s.to_string(),
                ScopeValue::List(list) => {
                    let mut buffer = String::new();
                    self.list_to_string(list, &mut buffer, node_id);
                    buffer
                }
            },
            None => self
                .state
                .get(&path, node_id)
                .and_then(|val| val.as_ref().try_into().ok())
                .unwrap_or_else(String::new),
        }
    }

    pub fn get_string_no_sub(&self, path: &Path) -> String {
        match self.scope.lookup(path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get_string_no_sub(path),
                ScopeValue::Static(s) => s.to_string(),
                ScopeValue::List(list) => {
                    let mut buffer = String::new();
                    self.list_to_string_no_sub(list, &mut buffer);
                    buffer
                }
            },
            None => self
                .state
                .get_no_sub(&path)
                .and_then(|val| val.as_ref().try_into().ok())
                .unwrap_or_else(String::new),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    type Sub = usize;

    #[test]
    fn scope_value() {
        let mut root = Scope::new(None);
        root.scope(
            "value".into(),
            Cow::Owned(ScopeValue::Static("hello world".into())),
        );

        let mut inner = Scope::new(Some(&root));
        let value = inner.lookup_parent(&"value".into()).unwrap();
        inner.scope("shadow".into(), Cow::Borrowed(value));

        let ScopeValue::Static(lhs) = inner.lookup(&"shadow".into()).unwrap() else {
            panic!()
        };
        let ScopeValue::Static(rhs) = inner.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let mut ctx = Context::new(&mut state, &mut root);
        let mut attributes = HashMap::new();
        attributes.insert(
            "name".to_string(),
            ScopeValue::Dyn(Path::Key("name".into())),
        );

        let id = 123.into();
        let name: Option<String> = ctx.attribute("name", &id, &attributes);

        assert_eq!("Dirk Gently", name.unwrap());
    }
}
