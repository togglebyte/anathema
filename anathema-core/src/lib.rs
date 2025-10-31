//! TODO: write docs
#![deny(missing_docs)]

pub mod frontend;

/// State aliases and imports.
// This is because all the value types in core should use the value index.
pub mod state {
    // use std::cell::RefCell;

    // pub use anathema_state::State;
    // pub use anathema_state::{Change, Type, TypeId};
    // // pub use anathema_state::{List, Map, State};
    // use anathema_store::stack::Stack;
    // use super::ValueIndex;

    // pub(crate) type Changes = Stack<(ValueIndex, Change)>;

    // // TODO: This has to be behind the cfg not(multithread)
    // thread_local! {
    //     static CHANGES: RefCell<Changes> = RefCell::new(Default::default());
    // }

    // /// Value alias
    // pub type Value<T> = anathema_state::Value<ValueIndex, T>;
    // /// Map alias
    // pub type Map<T> = anathema_state::Map<ValueIndex, T>;
    // /// List alias
    // pub type List<T> = anathema_state::List<ValueIndex, T>;

    // /// AnonValue alias
    // pub type AnonValue = anathema_state::AnonValue<ValueIndex>;
}

// Decide what to do here
// pub mod ui;
