use crate::hashmap::HashMap;
use crate::{Attributes, NodeId, Path, State, ValueRef};

#[derive(Debug)]
pub enum Value<T> {
    Static(T),
    /// Any value associated with the state is subject to change
    Cached {
        val: Option<T>,
        path: Path,
    },
    Empty,
}

impl<T> Value<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Static(val) => Some(val),
            Self::Cached { val, .. } => val.as_ref(),
            Self::Empty => None,
        }
    }

    pub fn reload(&mut self, state: &mut impl State) {
        panic!()
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    inner: HashMap<Path, ValueRef<'a>>,
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

    pub fn scope(&mut self, path: Path, value: ValueRef<'a>) {
        self.inner.insert(path, value);
    }

    pub fn lookup(&self, path: &Path) -> Option<ValueRef<'a>> {
        match self.inner.get(path) {
            Some(value) => Some(value.clone()),
            None => self.parent?.lookup(path),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Context<'a: 'val, 'val> {
    pub state: &'a dyn State,
    pub scope: &'a Scope<'val>,
}

impl<'a, 'val> Context<'a, 'val> {
    pub fn new(state: &'a dyn State, scope: &'a Scope<'val>) -> Self {
        Self { state, scope }
    }

    pub fn lookup_value<T>(&self, path: &Path, node_id: Option<&NodeId>) -> Value<T>
    where
        for<'b> T: TryFrom<ValueRef<'b>>,
    {
        // TODO: come back and unwack this one.
        //       the top two arms just differ because of the path, but the `Deferred` owns the path
        match self.scope.lookup(path) {
            Some(ValueRef::Deferred(path)) => Value::Cached {
                val: self
                    .state
                    .get(&path, node_id)
                    .and_then(|val_ref| T::try_from(val_ref).ok()),
                path: path.clone(),
            },
            None => Value::Cached {
                val: self
                    .state
                    .get(&path, node_id)
                    .and_then(|val_ref| T::try_from(val_ref).ok()),
                path: path.clone(),
            },
            Some(value_ref) => match T::try_from(value_ref) {
                Ok(val) => Value::Static(val),
                Err(_) => Value::Empty,
            },
        }
    }

    pub(super) fn lookup_old(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'a>> {
        self.scope
            .lookup(path)
            .or_else(|| self.state.get(path, node_id))
    }

    /// Lookup a value, if the value belongs to the state it returns a deferred value 
    /// instead, to be resolved at a later stage.
    pub(super) fn lookup(&self, path: &Path) -> Option<ValueRef<'a>> {
        self.scope
            .lookup(path)
            .or_else(|| Some(ValueRef::Deferred(path.clone())))
    }

    pub fn attribute<T: ?Sized>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'val Attributes,
    ) -> Value<T>
    where
        T: Clone,
        for<'b> T: TryFrom<ValueRef<'b>>,
    {
        let Some(value) = attributes.get(key.as_ref()) else {
            return Value::Empty;
        };
        value.resolve(self, node_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;
    use crate::ValueExpr;

    type Sub = usize;

    #[test]
    fn scope_value() {
        let mut scope = Scope::new(None);
        scope.scope("value".into(), ValueRef::Str("hello world"));

        let mut inner = scope.reparent();

        inner.scope("value".into(), ValueRef::Str("inner hello"));
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
        let ctx = Context::new(&mut state, &mut root);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let id = Some(123.into());
        let name = ctx.attribute::<String>("name", id.as_ref(), &attributes);
        assert_eq!("Dirk Gently", name.value().unwrap());
    }

    #[test]
    fn context_lookup() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);

        let path = Path::from("inner").compose("name");
        let value = context.lookup_value::<String>(&path, None);
        let value: &str = value.value().unwrap();
        assert!(matches!(value, "Fiddle McStick"));
    }

    #[test]
    fn singular_state_value() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);
        let path = Path::from("inner").compose("name");
    }

    #[test]
    fn collection_with_one_state_value() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);
    }
}
