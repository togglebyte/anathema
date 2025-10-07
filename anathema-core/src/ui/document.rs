use std::cell::RefCell;

use crate::attributes::AllAttributes;
use crate::elements::Element;
use crate::layout::Layout;
use crate::nodes::{InsertNode, NodeId, Nodes};

#[derive(Debug)]
pub struct Document<'a> {
    nodes: &'a mut Nodes,
    layout: &'a mut Layout,
    attributes: &'a mut AllAttributes,
}

impl<'a> Document<'a> {
    pub fn insert(&mut self, insert: InsertNode, parent: Option<NodeId>) -> NodeId {
        let node_id = self.nodes.apply_insert(insert, parent, self.layout, self.attributes);
        let node = self.nodes.node(node_id);
        node.layout(self.layout);

        if let Some(parent) = parent {
            self.nodes[parent].children.push(node_id);
        }

        node_id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::test_el;

    fn with_doc<F>(f: F)
    where
        F: Fn(Document<'_>),
    {
        let mut nodes = Nodes::empty();
        let mut layout = Layout::empty();
        let mut attributes = AllAttributes::empty();

        let doc = Document {
            nodes: &mut nodes,
            layout: &mut layout,
            attributes: &mut attributes,
        };

        f(doc);
    }

    #[test]
    fn insert_node() {
        with_doc(|mut doc| {
            let node = test_el("dad").add_child(test_el("baby"));
            let parent = doc.insert(node, None);
            doc.insert(test_el("other baby"), Some(parent));
            panic!("{doc:#?}");
        });
    }
}
