use crate::hashmap::IntMap;
use crate::ValueRef;

pub struct Map<T>(IntMap<usize, ValueRef<T>>);
