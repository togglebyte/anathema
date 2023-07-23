


mod generational;
mod simple;

pub(crate) type Index = usize;

pub(crate) use generational::GenerationSlab;
pub(crate) use simple::Slab;

