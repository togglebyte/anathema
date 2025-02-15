use anathema_debug::DebugWriter;
use anathema_store::store::{OwnedEntry, OwnedKey};

use super::subscriber::Subscribers;
use super::values::OwnedValue;
use super::{CHANGES, OWNED, SHARED, SUBSCRIBERS};
use crate::store::subscriber::SubscriberDebug;
use crate::Change;

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
            Change::Inserted(idx) => write!(output, "<inserted at {idx}>"),
            Change::Removed(idx) => write!(output, "<removed {idx}>"),
            Change::Dropped => write!(output, "<dropped>"),
            Change::Changed => write!(output, "<changed>"),
        }?;

        writeln!(output)
    }
}
