use std::rc::Rc;

use anathema_values::Path;

use crate::Value;

#[derive(Debug)]
pub struct Loop {
    binding: Path,
    collection: Box<[Value]>,
}

impl Loop {
    pub fn new(binding: Path, collection: Box<[Value]>) -> Self {
        Self {
            binding,
            collection,
        }
    }
}
