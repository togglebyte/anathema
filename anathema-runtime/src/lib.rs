// -----------------------------------------------------------------------------
//   - Runtime -
//   1. Creating the initial widget tree
//   2. Runtime loop >----------------------------------------+
//    ^  2.1. Wait for messages                               |
//    |  2.2. Wait for events                                 v
//    |  2.4. Was there events / messages / data changes? (no) (yes)
//    |                                                    |    |
//    +----------------------------------------------------+    |
//    |       +-------------------------------------------------+
//    |       |
//    |       V
//    |       1. Layout
//    |       2. Position
//    |       3. Draw
//    +-----< 4. Run again
//
// -----------------------------------------------------------------------------

use std::sync::atomic::AtomicBool;


pub use crate::builder::Builder;
pub use crate::error::{Error, Result};
pub use crate::runtime::{Frame, Runtime};

static REBUILD: AtomicBool = AtomicBool::new(false);

mod error;

pub mod builder;
mod events;
pub mod runtime;
