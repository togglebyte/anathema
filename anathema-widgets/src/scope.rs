use std::fmt::{self, Debug, Write};

use anathema_debug::DebugWriter;
use anathema_state::{Path, PendingValue, StateId, States};

use crate::expressions::{Downgraded, EvalValue};
use crate::values::ValueId;

#[derive(Debug)]
pub struct ScopeLookup<'bp> {
    path: Path<'bp>,
    id: ValueId,
}

impl<'bp> ScopeLookup<'bp> {
    /// Get and subscribe to a value
    pub(crate) fn new(path: impl Into<Path<'bp>>, value_id: ValueId) -> Self {
        Self {
            path: path.into(),
            id: value_id,
        }
    }
}

#[derive(Default)]
enum Entry<'bp> {
    /// Scope(size of previous scope)
    Scope(usize),
    Downgraded(Path<'bp>, Downgraded<'bp>),
    Pending(Path<'bp>, PendingValue),
    State(StateId),
    /// This is marking the entry as free, and another entry can be written here.
    /// This is not indicative of a missing value
    #[default]
    Empty,
}

impl<'bp> Entry<'bp> {
    fn get(&self, lookup: &ScopeLookup<'bp>) -> Option<&Self> {
        match self {
            Self::Downgraded(path, _) if *path == lookup.path => Some(self),
            Self::Pending(path, _) if *path == lookup.path => Some(self),
            Self::State(_) => Some(self),
            _ => None,
        }
    }
}

impl Debug for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Scope(scope) => f.debug_tuple("Scope").field(scope).finish(),
            Entry::Pending(path, pending_value) => f.debug_tuple("Pending").field(path).field(pending_value).finish(),
            Entry::Downgraded(path, value) => f.debug_tuple("Downgraded").field(path).field(value).finish(),
            Entry::State(state) => f.debug_tuple("State").field(&state).finish(),
            Entry::Empty => f.debug_tuple("Empty").finish(),
        }
    }
}

/// `Scope` should be created once and then re-used by the runtime
/// to avoid unnecessary allocations.
///
/// The scope is recreated for the update path of the nodes.
#[derive(Debug, Default)]
pub struct Scope<'bp> {
    storage: Vec<Entry<'bp>>,
    current_scope_size: usize,
    storage_index: usize,
    level: usize,
}

impl<'bp> Scope<'bp> {
    pub fn new() -> Self {
        Self {
            storage: vec![],
            current_scope_size: 0,
            storage_index: 0,
            level: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.storage_index
    }

    pub fn with_capacity(cap: usize) -> Self {
        let mut storage = Vec::with_capacity(cap);
        storage.fill_with(Default::default);
        Self {
            storage,
            current_scope_size: 0,
            storage_index: 0,
            level: 0,
        }
    }

    /// Clear the storage by writing `Entry::Empty` over every
    /// existing entry, and reset the storage index
    pub fn clear(&mut self) {
        self.storage[..self.storage_index].fill_with(Default::default);
        self.storage_index = 0;
    }

    fn insert_entry(&mut self, entry: Entry<'bp>) {
        match self.storage_index == self.storage.len() {
            true => self.storage.push(entry),
            false => self.storage[self.storage_index] = entry,
        }
        self.current_scope_size += 1;
        self.storage_index += 1;
    }

    fn inner_get(
        &self,
        lookup: &ScopeLookup<'bp>,
        offset: &mut Option<usize>,
        states: &States,
    ) -> Option<EvalValue<'bp>> {
        let mut current_offset = offset.unwrap_or(self.storage.len());

        loop {
            let (new_offset, entry) = self.storage[..current_offset]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, e)| e.get(lookup).map(|e| (i, e)))?;

            current_offset = new_offset;
            *offset = Some(new_offset);

            match entry {
                // Pending
                Entry::Pending(_, pending) => break Some(EvalValue::Dyn(pending.to_value(lookup.id))),

                // Downgraded
                Entry::Downgraded(_, downgrade) => break Some(downgrade.upgrade(lookup.id)),

                // State value
                &Entry::State(state_id) => {
                    let state = states.get(state_id)?;
                    if let Some(value) = state.state_get(lookup.path, lookup.id) {
                        break Some(EvalValue::Dyn(value));
                    }
                }
                _ => continue,
            }
        }
    }

    /// Get can never return an eval value that is downgraded or pending
    pub(crate) fn get(
        &self,
        lookup: ScopeLookup<'bp>,
        offset: &mut Option<usize>,
        states: &States,
    ) -> Option<EvalValue<'bp>> {
        self.inner_get(&lookup, offset, states)
    }

    pub fn insert_state(&mut self, state_id: StateId) {
        let entry = Entry::State(state_id);
        self.insert_entry(entry);
    }

    pub(crate) fn push(&mut self) {
        self.insert_entry(Entry::Scope(self.current_scope_size));
        self.current_scope_size = 0;
        self.level += 1;
    }

    pub(crate) fn pop(&mut self) {
        if self.storage_index == 0 {
            return;
        }

        let index = self.storage_index - 1 - self.current_scope_size;
        let &Entry::Scope(size) = &self.storage[index] else { panic!() };
        self.storage[index..].fill_with(|| Entry::Empty);
        self.storage_index = index;
        self.current_scope_size = size;
        self.level -= 1;
    }

    pub(crate) fn scope_pending(&mut self, key: &'bp str, iter_value: PendingValue) {
        let entry = Entry::Pending(Path::from(key), iter_value);
        self.insert_entry(entry);
    }

    pub(crate) fn scope_downgrade(&mut self, binding: &'bp str, downgrade: Downgraded<'bp>) {
        let entry = Entry::Downgraded(Path::from(binding), downgrade);
        self.insert_entry(entry);
    }
}

pub struct DebugScope<'a, 'b>(pub &'a Scope<'b>);

impl DebugWriter for DebugScope<'_, '_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        for (i, entry) in self.0.storage.iter().enumerate() {
            writeln!(output, "{i:02} {entry:?}")?;
        }
        Ok(())
    }
}

// #[cfg(test)]
impl<'bp> Scope<'bp> {
    pub fn debug(&self) -> String {
        let mut s = String::new();

        for (i, entry) in self.storage.iter().enumerate() {
            s += &format!("{i:02} {entry:?}\n");
        }

        s += "-----------------------\n";
        s += &format!(
            "current scope size: {} | level: {}\n",
            self.current_scope_size, self.level
        );

        s
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{List, Map, Subscriber, Value};
    use anathema_templates::{Expression, Globals};

    use super::*;
    use crate::expressions::eval_collection;
    use crate::testing::ScopedTest;

    #[test]
    fn fetch_state_value() {
        ScopedTest::new()
            .with_value("a", 123u32)
            .lookup(ScopeLookup::new("a", Subscriber::ZERO), |val| {
                let val = val.load::<u32>().unwrap();
                assert_eq!(val, 123u32);
            });
    }

    #[test]
    fn scope_collection() {
        let mut map = Map::<Value<List<u8>>>::empty();
        map.insert("list", Value::<List<u8>>::from_iter([1u8, 2, 3]));

        let states = States::new();
        let scope = Scope::new();
        let expr = Expression::Ident("list".into());
        let globals = Globals::new(Default::default());
        eval_collection(&expr, &globals, &scope, &states, ValueId::ZERO);

        //         let one = [Expression::Primitive(1i64.into())];

        //         scope.scope_pending("a", &one);
        //         scope.scope_downgraded("a", 0);

        //         scope.push();
        //         let two = [Expression::Primitive(2i64.into())];
        //         scope.scope_static_collection("a", &two); // |_ a = 2i64
        //         scope.scope_index_lookup("a", 0); // |
        //         scope.pop();

        //         let ScopeValue::Dyn(expr) = scope.get(ScopeLookup::lookup("a"), &mut None).unwrap() else {
        //             panic!()
        //         };
        //         let val = eval(expr, &scope, Subscriber::ZERO);
        //         assert_eq!(1, val.load_number().unwrap().as_int());
    }
}
