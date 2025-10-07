use anathema_state::{PendingValue, StateId};
use anathema_store::slab::Key;

use crate::expression::{Kind, ResolvedExpr};
use crate::{Collection, Value, ValueKind};

#[derive(Debug)]
pub(crate) enum Entry<'parent, 'bp> {
    Component { state: StateId, component_attributes: Key },
    Collection(&'parent Collection<'bp>),
    Index(&'bp str, usize, PendingValue),
    WithValue(&'bp str, &'parent Value<'bp>),
    Empty,
}

#[derive(Debug)]
pub struct Scope<'parent, 'bp> {
    outer: Option<&'parent Scope<'parent, 'bp>>,
    parent: Option<&'parent Scope<'parent, 'bp>>,
    value: Entry<'parent, 'bp>,
}

impl<'parent, 'bp> Scope<'parent, 'bp> {
    fn new(value: Entry<'parent, 'bp>) -> Self {
        Self {
            parent: None,
            outer: None,
            value,
        }
    }

    pub fn with_component(state: StateId, attributes: Key, outer: Option<&'parent Scope<'parent, 'bp>>) -> Self {
        Self {
            outer,
            parent: None,
            value: Entry::Component {
                state,
                component_attributes: attributes,
            },
        }
    }

    pub fn with_collection(collection: &'parent Collection<'bp>, parent: &'parent Scope<'parent, 'bp>) -> Self {
        let value = Entry::Collection(collection);
        Self {
            outer: None,
            parent: Some(parent),
            value,
        }
    }

    pub fn with_index(
        binding: &'bp str,
        index: usize,
        parent: &'parent Scope<'parent, 'bp>,
        loop_index: PendingValue,
    ) -> Self {
        let value = Entry::Index(binding, index, loop_index);
        Self {
            value,
            parent: Some(parent),
            outer: None,
        }
    }

    pub fn with_value(binding: &'bp str, value: &'parent Value<'bp>, parent: &'parent Scope<'parent, 'bp>) -> Self {
        let value = Entry::WithValue(binding, value);
        Self {
            outer: None,
            parent: Some(parent),
            value,
        }
    }

    pub fn root() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self::new(Entry::Empty)
    }

    pub(crate) fn get_state(&self) -> Option<StateId> {
        match &self.value {
            Entry::Component { state, .. } => Some(*state),
            _ => self.parent?.get_state(),
        }
    }

    pub(crate) fn get_attributes(&self) -> Option<Key> {
        match &self.value {
            Entry::Component {
                component_attributes, ..
            } => Some(*component_attributes),
            _ => self.parent?.get_attributes(),
        }
    }

    pub(crate) fn lookup(&self, key: &str) -> Option<ResolvedExpr<'bp>> {
        match self.value {
            Entry::WithValue(ident, val) if ident == key => Some(val.expr.clone()),
            Entry::Index(_, _, loop_index) if key == "loop" => Some(ResolvedExpr::Int(Kind::Dyn(loop_index))),
            Entry::Index(binding, index, _) if key == binding => {
                match self.parent.expect("the parent can only be a collection").value {
                    Entry::Collection(collection) => match &collection.0.kind {
                        ValueKind::List(_) => {
                            let value_expr = ResolvedExpr::Index(
                                collection.0.expr.clone().into(),
                                ResolvedExpr::Int(Kind::Static(index as i64)).into(),
                            );
                            Some(value_expr)
                        }
                        ValueKind::DynList(value) => {
                            let state = value.as_state()?;
                            let list = state.as_any_list()?;
                            let value = list.lookup(index)?;
                            Some(value.into())
                        }
                        &ValueKind::Range(from, to) => (from..to)
                            .skip(index)
                            .map(|num| ResolvedExpr::Int(Kind::Static(num as i64)))
                            .next(),
                        _ => unreachable!("none of the other values can be a collection"),
                    },
                    _ => unreachable!("the parent scope is always a collection"),
                }
            }
            _ => self.parent?.lookup(key),
        }
    }

    /// Get the outer scope
    ///
    /// # Panics
    ///
    /// This will panic if the outer scope has been set incorrectly
    /// or there is no parent scope.
    pub fn outer(&self) -> &'parent Scope<'parent, 'bp> {
        match self.outer {
            Some(scope) => scope,
            None => match self.parent {
                Some(parent) => parent.outer(),
                None => panic!("no outer scope, no parent"),
            },
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn scope_one() {
        // let mut scope = Scope::new();
        // panic!();
        // scope.scope("key",
    }
}
