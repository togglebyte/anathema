use std::cell::RefCell;

use crate::attributes::AllAttributes;
use crate::runtime::elements::{Element, ElementId, Elements};
use crate::layout::Layout;

#[derive(Debug)]
pub struct Document<'a, 'bp> {
    elements: &'a mut Elements<'bp>,
    layout: &'a mut Layout,
    attributes: &'a mut AllAttributes,
}

impl<'a, 'bp> Document<'a, 'bp> {
    pub fn insert(&mut self, insert: (), parent: Option<ElementId>) -> ElementId {
        panic!()
        // let node_id = self.elements.apply_insert(insert, parent, self.layout, self.attributes);
        // let node = self.elements.node(node_id);
        // node.layout(self.layout);

        // if let Some(parent) = parent {
        //     self.elements[parent].children.push(node_id);
        // }

        // node_id
    }

    pub(crate) fn new(elements: &'a mut Elements<'bp>, layout: &'a mut Layout, attributes: &'a mut AllAttributes) -> Self {
        Self {
            elements,
            layout,
            attributes,
        }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::testing::test_el;

    // fn with_doc<F>(f: F)
    // where
    //     F: Fn(Document<'_>),
    // {
    //     let mut nodes = Elements::empty();
    //     let mut layout = Layout::empty();
    //     let mut attributes = AllAttributes::empty();

    //     let doc = Document {
    //         elements: &mut nodes,
    //         layout: &mut layout,
    //         attributes: &mut attributes,
    //     };

    //     f(doc);
    // }

    // #[test]
    // fn insert_node() {
    //     with_doc(|mut doc| {
    //         let node = test_el("dad").add_child(test_el("baby"));
    //         let parent = doc.insert(node, None);
    //         doc.insert(test_el("other baby"), Some(parent));
    //         panic!("{doc:#?}");
    //     });
    // }
}
