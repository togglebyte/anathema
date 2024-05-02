pub use self::owned::{Owned, OwnedEntry, OwnedKey};
pub use self::shared::{Shared, SharedKey};
use crate::slab::{RcSlab, Slab};

mod owned;
mod shared;
