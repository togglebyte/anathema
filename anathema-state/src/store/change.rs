use anathema_store::stack::Stack;

use super::subscriber::{SubKey, Subscribers};
use super::{CHANGES, SUBSCRIBERS};
use crate::PendingValue;

pub type Changes = Stack<(Subscribers, Change)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    // TODO Given that value store will keep at most u32::MAX
    //      values it would stand to reason that both `Inserted` and `Removed`
    //      can use u32 instead of usize to save a bit of space.
    Inserted(usize, PendingValue),
    Removed(usize),
    Changed,
    Dropped,
}

/// Drain and iterate over changes with the associated subscribers,
/// and applies `F` to each subscriber + &change.
///
/// NOTE: A singular `Subscriber` can contain multible keys.
pub fn drain_changes(local_changes: &mut Changes) {
    CHANGES.with_borrow_mut(|changes| changes.drain_into(local_changes));
}

pub(crate) fn changed(subkey: SubKey, change: Change) {
    let subscribers = SUBSCRIBERS.with_borrow(|subs| subs.get(subkey));
    if subscribers.is_empty() {
        return;
    }
    CHANGES.with_borrow_mut(|changes| changes.push((subscribers, change)));
}
