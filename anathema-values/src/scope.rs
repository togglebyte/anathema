use std::borrow::Cow;
use std::fmt::{self, Display, Write};
use std::ops::{Add, Deref, Div, Mul, Rem, Sub};
use std::rc::Rc;
use std::str::FromStr;

use anathema_render::{Color, Size, Style};

use crate::hashmap::HashMap;
use crate::{NodeId, Path, State, Value, ValueExpr, ValueRef};

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

    /// Increase the length of a state collection.
    /// This is a manual step for state bound lists
    /// as we don't access the entire list, only
    /// one value at a time when needed.
    pub fn add(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
        }
    }

    /// Decrease the length of a state collection.
    /// This is a manual step (see `Self::add`)
    pub fn remove(&mut self) {
        if let Collection::State { len, .. } = self {
            *len -= 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeValue {
    /// Static values are "cheap" to clone / copy.
    Static(Value),
    // Expr(ValueExpr),
    // List(Rc<[ScopeValue]>),
    Dyn(Path),
    Invalid,
}

// TODO: do we even need this? - 2023-09-26
// impl<const N: usize> From<[ScopeValue; N]> for ScopeValue {
//     fn from(arr: [ScopeValue; N]) -> Self {
//         if N == 1 {
//             arr.into_iter()
//                 .next()
//                 .expect("this is always going to be an array with a size of one")
//         } else {
//             ScopeValue::List(Rc::new(arr))
//         }
//     }
// }

// TODO: add a testing flag for this
impl From<String> for ScopeValue {
    fn from(s: String) -> Self {
        Self::Static(Value::Str(s.into()))
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    inner: HashMap<Path, Cow<'a, ScopeValue>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            parent,
            inner: HashMap::new(),
        }
    }

    pub fn scope(&mut self, path: Path, value: Cow<'a, ScopeValue>) {
        self.inner.insert(path, value);
    }

    pub fn reparent(&self) -> Scope<'_> {
        Scope::new(Some(self))
    }

    // /// Scope a value for a collection.
    // /// TODO: Review if the whole cloning business here makes sense
    // pub fn scope_collection(&mut self, binding: Path, collection: &Collection, value_index: usize) {
    //     let value = match collection {
    //         Collection::Rc(list) => Cow::Owned(list[value_index].clone()),
    //         Collection::State { path, .. } => {
    //             let path = path.compose(value_index);
    //             Cow::Owned(ScopeValue::Dyn(path))
    //         }
    //         Collection::Empty => return,
    //     };

    //     self.scope(binding, value);
    // }

    pub fn lookup(&self, path: &Path) -> Option<&ScopeValue> {
        self.inner
            .get(path)
            .map(Deref::deref)
            .or_else(|| self.parent.and_then(|parent| parent.lookup(path)))
    }

    // pub fn lookup_list(&self, path: &Path) -> Option<Rc<[ScopeValue]>> {
    //     self.lookup(path).and_then(|value| match value {
    //         ScopeValue::List(list) => Some(list.clone()),
    //         _ => None,
    //     })
    // }
}

pub struct Context<'a: 'val, 'val> {
    pub state: &'a dyn State,
    pub scope: &'a Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a dyn State, scope: &'a mut Scope<'val>) -> Self {
        Self { state, scope }
    }

    /// Resolve a value based on paths.
    pub fn resolve(&self, value: &ScopeValue) -> ScopeValue {
        // TODO toodles
        panic!()
        // match value {
        //     ScopeValue::Static(_) => value.clone(),
        //     ScopeValue::Dyn(path) => match self.scope.lookup(path) {
        //         Some(lark @ ScopeValue::Dyn(p)) => self.resolve(lark),
        //         Some(_) => value.clone(),
        //         None => ScopeValue::Dyn(path.clone()),
        //     },
        //     ScopeValue::List(list) => {
        //         let values = list.iter().map(|v| self.resolve(v)).collect();
        //         ScopeValue::List(values)
        //     }
        // }
    }

    //     pub fn get_scope(&mut self, path: &Path, node_id: Option<&NodeId>) -> Option<Value> {
    //         match self.scope.lookup(&path).cloned() {
    //             Some(ScopeValue::Static(val)) => Some(val),
    //             Some(val) => match val {
    //                 ScopeValue::Static(val) => Some(val),
    //                 ScopeValue::Expr(expr) => expr.eval(self, node_id),
    //                 ScopeValue::Invalid => panic!("lol"),
    //             }
    //             None => self
    //                 .state
    //                 .get(&path, node_id.into())
    //                 .map(|val| val.into_owned())
    //         }
    //     }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<T: ?Sized>(&self, path: &Path, node_id: Option<&NodeId>) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        match self.scope.lookup(&path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get(path, node_id),
                ScopeValue::Static(s) => <&T>::try_from(s).ok(),
                ScopeValue::Invalid => None,
            },
            None => self
                .state
                .get(&path, node_id.into())
                .and_then(|val| val.try_into().ok()),
        }
    }

    pub fn attribute<T: ?Sized>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'val HashMap<String, ScopeValue>,
    ) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        let attrib = attributes.get(key.as_ref())?;

        match attrib {
            ScopeValue::Static(val) => val.try_into().ok(),
            ScopeValue::Dyn(path) => self.get(path, node_id),
            _ => None,
        }
    }

    pub fn primitive<U>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<U>
    where
        U: for<'b> TryFrom<&'b Value>,
    {
        panic!()
        // let attrib = attributes.get(key.as_ref())?;

        // match attrib {
        //     ScopeValue::Static(val) => T::try_from(val).ok(),
        //     ScopeValue::Dyn(path) => self.get::<T>(path, node_id),
        //     _ => None,
        // }
    }

    pub fn list_to_string(
        &self,
        list: &Rc<[ScopeValue]>,
        buffer: &mut String,
        node_id: Option<&NodeId>,
    ) {
        panic!()
        // for val in list.iter() {
        //     match val {
        //         ScopeValue::List(list) => self.list_to_string(list, buffer, node_id),
        //         ScopeValue::Dyn(path) => buffer.push_str(&self.get_string(path, node_id)),
        //         ScopeValue::Static(s) => drop(write!(buffer, "{s}")),
        //     }
        // }
    }

    pub fn get_string(&self, path: &Path, node_id: Option<&NodeId>) -> String {
        panic!()
        // match self.scope.lookup(path) {
        //     Some(val) => match val {
        //         ScopeValue::Dyn(path) => self.get_string(path, node_id),
        //         ScopeValue::Static(s) => s.to_string(),
        //         ScopeValue::List(list) => {
        //             let mut buffer = String::new();
        //             self.list_to_string(list, &mut buffer, node_id);
        //             buffer
        //         }
        //     },
        //     None => self
        //         .state
        //         .get(&path, node_id)
        //         .map(|val| val.to_string())
        //         .unwrap_or_else(String::new),
        // }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    type Sub = usize;

    #[test]
    fn scope_value() {
        let mut scope = Scope::new(None);
        scope.scope("value".into(), Cow::Owned("hello world".to_string().into()));

        let mut inner = scope.reparent();

        inner.scope("value".into(), Cow::Owned("inner hello".to_string().into()));
        let value = inner.lookup(&"value".into()).unwrap();

        let ScopeValue::Static(Value::Str(lhs)) = value else { panic!() };
        assert_eq!(&**lhs, "inner hello");

        let ScopeValue::Static(Value::Str(lhs)) = scope.lookup(&"value".into()).unwrap() else { panic!() }; 
        assert_eq!(&**lhs, "hello world");
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let mut ctx = Context::new(&mut state, &mut root);
        let mut attributes: HashMap<String, ScopeValue> = HashMap::new();
        attributes.insert(
            "name".to_string(),
            ScopeValue::Dyn(Path::Key("name".into())),
        );

        let id = Some(123.into());
        let name: &str = ctx.attribute("name", id.as_ref(), &attributes).unwrap();
        assert_eq!("Dirk Gently", name);
    }
}
