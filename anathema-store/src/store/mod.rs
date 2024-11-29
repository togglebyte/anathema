pub use self::owned::{Owned, OwnedEntry, OwnedKey};
pub use self::shared::{Shared, SharedKey};
// use crate::slab::{GenSlab, RcSlab, SharedSlab, Slab};

mod owned;
mod shared;
