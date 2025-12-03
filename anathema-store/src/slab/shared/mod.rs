pub(super) mod arc;
pub(super) mod rc;

/// TODO: document this trait
/// TODO: ignore previous todo, remove this as we are aiming for a generic trait for all slabs
pub trait Shared<I, T> {
    /// The shared container
    type Element;

    /// This will clone the underlying container (e.g Rc or Arc).
    /// Unlike the basic `Slab` the `SharedSlab` needs the values to be
    /// manually removed with `try_remove`.
    fn get(&mut self, index: I) -> Option<Self::Element>;

    /// Insert a value into the slab
    fn insert(&mut self, value: T) -> I;

    /// Take a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied.
    fn try_remove(&mut self, index: I) -> Option<T>;
}
