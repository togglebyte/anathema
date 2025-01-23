use anathema_debug::DebugWriter;
use anathema_store::store::{OwnedEntry, OwnedKey};

use super::subscriber::Subscribers;
use super::values::OwnedValue;
use super::{CHANGES, OWNED, SHARED, SUBSCRIBERS};
use crate::states::OldAnyState;
use crate::store::subscriber::SubscriberDebug;
use crate::Change;

// -----------------------------------------------------------------------------
//   - Owne value debug -
// -----------------------------------------------------------------------------
struct OwnedStateDebug<'a>(OwnedKey, &'a OwnedEntry<OwnedValue>);

impl DebugWriter for OwnedStateDebug<'_> {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        let key: usize = self.0.debug_index();
        panic!("do we need this at all?");
        // match self.1 {
        //     OwnedEntry::Occupied(state) => match state.val.to_common() {
        //         Some(val) => writeln!(output, "[{key}] : {val:?}"),
        //         None => writeln!(output, "[{key}] : <state>"),
        //     },
        //     OwnedEntry::Unique => writeln!(output, "[{key}] : <unique>"),
        //     OwnedEntry::Shared(k) => writeln!(output, "[{key}] : <shared {k:?}>"),
        // }
    }
}

// -----------------------------------------------------------------------------
//   - Shared value debug -
// -----------------------------------------------------------------------------
struct SharedStateDebug<'a>(usize, &'a OwnedValue);

impl DebugWriter for SharedStateDebug<'_> {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        panic!("is this relevant still?");
        // match self.1.val.to_common() {
        //     Some(val) => writeln!(output, "[{}] : {val:?}", self.0),
        //     None => writeln!(output, "[{}] : <state>", self.0),
        // }
    }
}

// -----------------------------------------------------------------------------
//   - Change debug -
// -----------------------------------------------------------------------------
struct ChangeDebug<'a>(&'a Subscribers, Change);

impl DebugWriter for ChangeDebug<'_> {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        // Subscribers
        self.0.iter().map(SubscriberDebug).for_each(|mut sub| {
            sub.write(output).unwrap();
            write!(output, ", ").unwrap();
        });

        write!(output, " - ")?;

        // Change
        match self.1 {
            Change::Inserted(idx, pending) => write!(
                output,
                "<inserted at {idx} | value {}>",
                pending.owned_key().debug_index()
            ),
            Change::Removed(idx) => write!(output, "<removed {idx}>"),
            Change::Dropped => write!(output, "<dropped>"),
            Change::Changed => write!(output, "<changed>"),
        }?;

        writeln!(output)
    }
}

/// Debug output of OWNED store value.
pub struct DebugOwnedStore;

impl DebugWriter for DebugOwnedStore {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        OWNED.with(|storage| {
            storage.for_each(|k, v| {
                OwnedStateDebug(k, v).write(output).unwrap();
                writeln!(output).unwrap();
            });
        });

        Ok(())
    }
}

/// Debug output of SHARED store value.
pub struct DebugSharedStore;

impl DebugWriter for DebugSharedStore {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        SHARED.with(|storage| {
            storage.for_each(|k, v| {
                SharedStateDebug(k, v).write(output).unwrap();
            });
        });

        Ok(())
    }
}

/// Debug output of SUBSCRUBERS store value.
pub struct DebugSubscribers;

impl DebugWriter for DebugSubscribers {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        SUBSCRIBERS.with_borrow(|storage| {
            for (k, v) in storage.inner.iter() {
                writeln!(output, "key: {k:?}:").unwrap();
                for sub in v.iter() {
                    SubscriberDebug(sub).write(output).unwrap();
                }
                writeln!(output).unwrap();
            }
        });

        Ok(())
    }
}

/// Debug output of CHANGES
pub struct ChangesDebug;

impl DebugWriter for ChangesDebug {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        CHANGES.with_borrow(|changes| {
            changes
                .iter()
                .for_each(|(subscribers, change)| ChangeDebug(subscribers, *change).write(output).unwrap())
        });

        Ok(())
    }
}
