pub use self::owned::{Owned, OwnedEntry};
pub use self::shared::{Shared, SharedKey};
use crate::slab::{GenSlab, SharedSlab};

mod owned;
mod shared;

pub trait DataStore<I> {
    type Slab: SharedSlab<I, Self>
    where
        Self: Sized;

    fn owned_access<F, U>(f: F) -> U
    where
        F: FnOnce(&mut GenSlab<OwnedEntry<Self>>) -> U,
        Self: Sized;

    fn shared_access<F, U>(f: F) -> U
    where
        F: FnOnce(&mut Self::Slab) -> U,
        Self: Sized;
}
