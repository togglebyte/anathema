use anathema_state::State;
use anathema_store::slab::{Slab, SlabIndex};

use crate::component::AnyComponent;

// -----------------------------------------------------------------------------
//   - Component Id -
// -----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComponentId(u32);

impl SlabIndex for ComponentId {
    const MAX: usize = u32::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
    }
}

type FnComp = Box<dyn Fn() -> Box<dyn AnyComponent>>;
type FnState = Box<dyn Fn() -> Box<dyn State>>;

enum Entry {
    Component(Box<dyn AnyComponent>, Box<dyn State>),
    PrototypeInstance(Box<dyn AnyComponent>, Box<dyn State>),
    Prototype(FnComp, FnState),
}

// -----------------------------------------------------------------------------
//   - Components -
// -----------------------------------------------------------------------------

pub struct Components {
    inner: Slab<ComponentId, Entry>,
}

impl Components {
    pub fn empty() -> Self {
        Self { inner: Slab::empty() }
    }
}
