use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::{Attributes, NodeId, Path, State, Value, ValueExpr, ValueRef};

#[derive(Debug)]
pub enum Collection {
    Rc(Rc<[ValueExpr]>),
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

#[derive(Debug, Clone)]
pub enum ScopeValue<'a> {
    Static(ValueRef<'a>),
    Dyn(&'a Path),
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

#[derive(Debug)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    inner: HashMap<Path, ScopeValue<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            parent,
            inner: HashMap::new(),
        }
    }

    pub fn reparent(&self) -> Scope<'_> {
        Scope::new(Some(self))
    }

    pub fn scope(&mut self, path: Path, value: ScopeValue<'a>) {
        self.inner.insert(path, value);
    }

    pub fn lookup(&self, path: &Path) -> Option<ValueRef<'a>> {
        match self.inner.get(path) {
            Some(ScopeValue::Static(value)) => Some(*value),
            Some(ScopeValue::Dyn(path)) => self.lookup(path),
            None => self.parent?.lookup(path),
        }
    }
}

pub struct Context<'a: 'val, 'val> {
    pub state: &'a dyn State,
    pub scope: &'a Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a dyn State, scope: &'a Scope<'val>) -> Self {
        Self { state, scope }
    }

    pub fn lookup(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'a>> {
        self.scope
            .lookup(path)
            .or_else(|| self.state.get(path, node_id))
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<T: ?Sized>(&self, path: &Path, node_id: Option<&NodeId>) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        self.lookup(path, node_id)
            .and_then(|value_ref| <&T>::try_from(value_ref).ok())
    }

    pub fn attribute<T: ?Sized>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'val Attributes,
    ) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        attributes
            .get(key.as_ref())
            .and_then(|expr| expr.eval(self, node_id))
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
        scope.scope(
            "value".into(),
            ScopeValue::Static(ValueRef::Str("hello world")),
        );

        let mut inner = scope.reparent();

        inner.scope(
            "value".into(),
            ScopeValue::Static(ValueRef::Str("inner hello")),
        );
        let ValueRef::Str(lhs) = inner.lookup(&"value".into()).unwrap() else {
            panic!()
        };

        assert_eq!(lhs, "inner hello");

        let ValueRef::Str(lhs) = scope.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "hello world");
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let mut ctx = Context::new(&mut state, &mut root);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let id = Some(123.into());
        let name: &str = ctx.attribute("name", id.as_ref(), &attributes).unwrap();
        assert_eq!("Dirk Gently", name);
    }

    #[test]
    fn context_lookup() {
        let state = TestState::new();
        let mut scope = Scope::new(None);
        let context = Context::new(&state, &scope);

        let path = Path::from("inner").compose("name");
        let value = context.lookup(&path, None).unwrap();
        panic!("{value:#?}");
    }
}
