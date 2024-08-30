use anathema_store::stack::Stack;

use super::subscriber::{SubKey, Subscribers};
use super::{CHANGES, SUBSCRIBERS};
use crate::PendingValue;

pub type Changes = Stack<(Subscribers, Change)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    Inserted(u32, PendingValue),
    Removed(u32),
    Changed,
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

pub(crate) fn changed(subkey: SubKey, change: Change) {
    let subscribers = SUBSCRIBERS.with_borrow(|subs| subs.get(subkey));
    if subscribers.is_empty() {
        return;
    }
    CHANGES.with_borrow_mut(|changes| {
        changes.push((subscribers, change));
    });
}
