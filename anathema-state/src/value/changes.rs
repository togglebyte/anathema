use anathema_store::slab::Key;
use anathema_store::stack::Stack;

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
