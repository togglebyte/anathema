use std::borrow::Cow;

use super::store::Store;
use super::ValueRef;
use crate::template::Template;
use crate::Value;

pub enum Expression<'tpl: 'parent, 'parent> {
    Node(&'tpl Template),
    For {
        body: &'tpl [Template],
        binding: &'parent str,
        collection: &'parent [Value],
    },
    Block(&'tpl [Template]),
}

impl<'tpl, 'parent> Expression<'tpl, 'parent> {
    pub fn node(template: &'tpl Template) -> Self {
        Self::Node(template)
    }

    pub fn for_loop(
        body: &'tpl [Template],
        binding: &'parent str,
        collection: &'parent [Value],
    ) -> Self {
        Self::For {
            body,
            binding,
            collection,
        }
    }
}
