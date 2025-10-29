//! TODO: write docs
#![deny(missing_docs)]
#[allow(unused_extern_crates)]
extern crate anathema_state as anathema;

pub mod attributes;
pub mod frontend;
pub mod layout;
pub mod runtime;
pub mod templates;

/// State aliases and imports.
// This is because all the value types in core should use the value index.
pub mod state {
    use std::cell::RefCell;

    use anathema::Change;
    use anathema_store::stack::Stack;

    use crate::runtime::ValueIndex;

    pub(crate) type Changes = Stack<(ValueIndex, Change)>;

    // TODO: This has to be behind the cfg not(multithread)
    thread_local! {
        static CHANGES: RefCell<Changes> = RefCell::new(Default::default());
    }

    /// Value alias
    pub type Value<T> = anathema_state::Value<ValueIndex, T>;
    /// Map alias
    pub type Map<T> = anathema_state::Map<ValueIndex, T>;
    /// List alias
    pub type List<T> = anathema_state::List<ValueIndex, T>;

    /// AnonValue alias
    pub type AnonValue = anathema_state::AnonValue<ValueIndex>;
}

// Decide what to do here
// pub mod ui;

pub(crate) mod testing;
