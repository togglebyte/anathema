use std::cell::RefCell;

use anathema_store::stack::Stack;
use anathema_store::store::{Monitor, Owned, OwnedKey, Shared};
use values::OwnedValue;
pub use watchers::Watched;
use watchers::{Watcher, Watchers};

use crate::Type;

pub(crate) use self::change::changed;
pub use self::change::{clear_all_changes, drain_changes, Change, Changes};
pub use self::subscriber::{FutureValues, Subscriber};
use self::subscriber::{SubKey, SubscriberMap};

mod change;
pub mod debug;
pub(crate) mod subscriber;
pub(crate) mod values;
pub(crate) mod watchers;

thread_local! {
    static OWNED: Owned<OwnedValue> = const { Owned::empty() };
    static SHARED: Shared<OwnedValue> = const { Shared::empty() };
    static SUBSCRIBERS: RefCell<SubscriberMap> = const { RefCell::new(SubscriberMap::empty()) };
    static CHANGES: RefCell<Changes> = const { RefCell::new(Stack::empty()) };
    static FUTURE_VALUES: RefCell<FutureValues> = const { RefCell::new(Stack::empty()) };
    static WATCHERS: RefCell<Watchers> = const { RefCell::new(Watchers::new()) };
    static WATCH_QUEUE: RefCell<Stack<Watcher>> = const { RefCell::new(Stack::empty()) };
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// A value key is a composite from an owned key and a sub key.
pub struct ValueKey(OwnedKey, SubKey);

impl ValueKey {
    pub fn owned(&self) -> OwnedKey {
        self.0
    }

    pub(crate) fn sub(&self) -> SubKey {
        self.1
    }

    pub fn type_info(&self) -> Type {
        let type_info = self.0.aux();
        match type_info {
            1 => Type::Int,
            2 => Type::Float,
            3 => Type::Char,
            4 => Type::String,
            5 => Type::Bool,
            6 => Type::Map,
            7 => Type::List,
            8 => Type::Composite,
            _ => unreachable!("corrupt type information")
        }
    }
}

/// Register a slab key that has an interest in a future value.
pub fn register_future(sub: Subscriber) {
    FUTURE_VALUES.with_borrow_mut(|futures| futures.push(sub));
}

/// Drain values from FUTURE_VALUES into the local stack.
pub fn drain_futures(local: &mut Stack<Subscriber>) {
    FUTURE_VALUES.with_borrow_mut(|futures| futures.drain_copy_into(local));
}

/// Clear all FUTURE_VALUES
pub fn clear_all_futures() {
    FUTURE_VALUES.with_borrow_mut(|futures| futures.clear());
}

/// Drain values from WATCH_QUEUE into the local stack.
pub fn drain_watchers(local: &mut Stack<Watcher>) {
    WATCH_QUEUE.with_borrow_mut(|watchers| watchers.drain_copy_into(local));
}

/// Remove all subscribers from values.
///
/// This keeps the values intact and leaves them with
/// empty subscribers.
pub fn clear_all_subs() {
    SUBSCRIBERS.with_borrow_mut(|subs| subs.clear_subscribers());
}

// -----------------------------------------------------------------------------
//   - Test functions -
// -----------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod testing {
    use crate::{Change, Subscriber};

    pub fn drain_changes() -> Vec<(Vec<Subscriber>, Change)> {
        let mut ret = vec![];

        super::CHANGES.with_borrow_mut(|changes| {
            changes.drain().for_each(|(subscribers, change)| {
                ret.push((subscribers.iter().collect(), change));
            });
            changes.clear();
        });

        ret
    }
}

#[cfg(test)]
mod test {
    use anathema_store::slab::SlabIndex;
    use test::values::new_value;

    use super::*;

    #[test]
    fn store_value() {
        let value = Box::new(0usize);
        let key = new_value(value);
        assert_eq!(key.owned(), OwnedKey::ZERO);
        assert_eq!(key.sub(), SubKey::from_usize(0));
    }
}
