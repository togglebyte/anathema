use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::runtime::elements::ElementId;
use crate::runtime::eval::values::TemplateValue;

#[derive(Debug)]
pub struct AllAttributes<'bp> {
    attributes: SecondaryMap<ElementId, Attributes<'bp>>,
}

impl<'bp> AllAttributes<'bp> {
    pub(crate) fn empty() -> Self {
        Self {
            attributes: SecondaryMap::empty(),
        }
    }

    pub(crate) fn insert(&mut self, id: ElementId, attributes: Attributes<'bp>) {
        self.attributes.insert(id, attributes);
    }
}

#[derive(Debug)]
pub struct Attributes<'bp> {
    inner: SmallMap<SmallIndex, TemplateValue<'bp>>,
}

impl<'bp> Attributes<'bp> {
    pub(crate) fn empty() -> Self {
        Self {
            inner: SmallMap::empty(),
        }
    }

    pub(crate) fn get(&self, s: &str) -> TemplateValue<'bp> {
        todo!()
    }
}
