use std::cell::RefCell;

use anathema_store::stack::Stack;
use anathema_store::store::{Owned, OwnedKey, Shared};

pub(crate) use self::change::changed;
pub use self::change::{clear_all_changes, drain_changes, Change, Changes};
pub use self::subscriber::{FutureValues, Subscriber};
use self::subscriber::{SubKey, SubscriberMap};
use crate::states::AnyState;

mod change;
pub mod debug;
pub(crate) mod subscriber;
pub(crate) mod values;

thread_local! {
    static OWNED: Owned<Box<dyn AnyState>> = const { Owned::empty() };
    static SHARED: Shared<Box<dyn AnyState>> = const { Shared::empty() };
    static SUBSCRIBERS: RefCell<SubscriberMap> = const { RefCell::new(SubscriberMap::empty()) };
    static CHANGES: RefCell<Changes> = const { RefCell::new(Stack::empty()) };
    static FUTURE_VALUES: RefCell<FutureValues> = const { RefCell::new(Stack::empty()) };
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
    use test::values::new_value;

    use super::*;

    #[test]
    fn store_value() {
        let value = Box::new(0usize);
        let key = new_value(value);
        assert_eq!(key.owned(), 0.into());
        assert_eq!(key.sub(), 0.into());
    }
}
