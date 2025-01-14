use anathema_state::StateId;
use anathema_store::slab::Key;

pub enum Lookup {
    State(StateId),
    ComponentProperties(Key),
}

enum Entry {
    State(StateId),
}

pub struct Scope {
    storage: Vec<Entry>,
    current_scope_size: usize,
    storage_index: usize,
    level: usize,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            storage: vec![],
            current_scope_size: 0,
            storage_index: 0,
            level: 0,
        }
    }

    pub fn insert_state(&mut self, state_id: StateId) {
        let entry = Entry::State(state_id);
        self.insert_entry(entry);
    }

    fn insert_entry(&mut self, entry: Entry) {
        match self.storage_index == self.storage.len() {
            true => self.storage.push(entry),
            false => self.storage[self.storage_index] = entry,
        }
        self.current_scope_size += 1;
        self.storage_index += 1;
    }

    pub(crate) fn get_state(&self) -> StateId {
        let e = self
            .storage
            .iter()
            .rev()
            .find(|e| matches!(e, Entry::State(_)))
            .expect("scope should always have at least one state");
        let Entry::State(id) = e else { unreachable!() };
        *id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_one() {
        // let mut scope = Scope::new();
        // panic!();
        // scope.scope("key", 
    }
}
