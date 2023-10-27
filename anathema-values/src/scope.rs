use crate::hashmap::HashMap;
use crate::{Attributes, NodeId, Path, State, ValueRef};

#[derive(Debug)]
pub enum Value<T> {
    Static(T),
    Cached { val: Option<T>, path: Path },
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

    pub fn lookup(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'a>> {
        self.scope
            .lookup(path)
            .or_else(|| self.state.get(path, node_id))
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
        let Some(value) = attributes.get(key.as_ref()) else { return Value::Empty };
        panic!()
        // let value_ref = value.eval_value(self, node_id)?;
        // T::try_from(value_ref).ok()
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
        scope.scope(
            "value".into(),
            ValueRef::Str("hello world"),
        );

        let mut inner = scope.reparent();

        inner.scope(
            "value".into(),
            ValueRef::Str("inner hello"),
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
        let ctx = Context::new(&mut state, &mut root);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let id = Some(123.into());
        let name: String = ctx.attribute("name", id.as_ref(), &attributes).unwrap();
        assert_eq!("Dirk Gently", name);
    }

    #[test]
    fn context_lookup() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);

        let path = Path::from("inner").compose("name");
        let value = context.lookup(&path, None).unwrap();
        assert!(matches!(value, ValueRef::Str("Fiddle McStick")));
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
