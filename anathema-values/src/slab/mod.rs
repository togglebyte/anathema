mod generational;
mod simple;

pub(crate) type Idx = usize;

pub(crate) use generational::GenerationSlab;
pub use simple::Slab;

