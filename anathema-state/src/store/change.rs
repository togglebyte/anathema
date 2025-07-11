use anathema_store::stack::Stack;

use super::subscriber::Subscribers;
use super::{CHANGES, SUBSCRIBERS, ValueKey};

pub type Changes = Stack<(Subscribers, Change)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    /// A value was inserted into a list
    Inserted(u32),
    /// A value was removed from a list
    Removed(u32),
    /// A value has changed
    Changed,
    /// Value was removed (e.g removed from a map)
    Dropped,
}

/// Drain the current changes into a local value.
pub fn drain_changes(local_changes: &mut Changes) {
    CHANGES.with_borrow_mut(|changes| changes.drain_into(local_changes));
}

/// Clear all changes
pub fn clear_all_changes() {
    CHANGES.with_borrow_mut(|changes| changes.clear());
}

pub(crate) fn changed(key: ValueKey, change: Change) {
    // Notify subscribers
    let subscribers = SUBSCRIBERS.with_borrow(|subs| subs.get(key.sub()));
    if subscribers.is_empty() {
        return;
    }

    CHANGES.with_borrow_mut(|changes| {
        changes.push((subscribers, change));
    });
}
