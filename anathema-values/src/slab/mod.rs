


mod generational;
mod simple;

pub(crate) type Idx = usize;

pub(crate) use generational::GenerationSlab;
pub(crate) use simple::Slab;

