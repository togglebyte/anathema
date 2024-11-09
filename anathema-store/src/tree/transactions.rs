use super::{Tree, TreeView};
use crate::slab::Key;
use crate::tree::nodepath::new_node_path;
use crate::tree::AsNodePath;

pub struct InsertTransaction<'a, 'tree, T> {
    tree: &'a mut TreeView<'tree, T>,
    node_id: Key,
    source: &'a [u16],
}

impl<'a, 'tree, T> InsertTransaction<'a, 'tree, T> {
    pub fn new(tree: &'a mut TreeView<'tree, T>, source: &'a [u16]) -> Self {
        let node_id = tree.values.next_id();
        Self { tree, node_id, source }
    }

    pub fn node_id(&self) -> Key {
        self.node_id
    }

    /// Insert a child under a given parent.
    /// This will return `None` if the parent does not exist
    pub fn commit_child(self, value: T) -> Option<Key> {
        assert_eq!(self.tree.offset, &self.source[..self.tree.offset.len()]);
        let relative = &self.source[self.tree.offset.len()..];

        let node_id = self.tree.layout.with_mut(relative, |nodes| {
            // The node path is the source + len of children in source
            let node_path = new_node_path(self.source, nodes.len() as u16);

            let node_id = self.tree.values.insert((node_path, value));
            nodes.push(node_id);
            node_id
        })?;

        debug_assert_eq!(node_id, self.node_id);
        Some(self.node_id)
    }

    /// Insert a node at a given path.
    /// This will force all the values **after** the new node
    /// (along with all the children of the values) to have their paths updated.
    pub fn commit_at(mut self, value: T) -> Option<Key> {
        let (parent, index) = self.source.split_parent()?;

        let node_id = self.tree.layout.with_mut(parent, |siblings| {
            let value_id = self.tree.values.insert((self.source.into(), value));

            // Insert value id at a given index...
            siblings.insert(index, value_id);

            // ... and update the path to the succeeding siblings
            siblings.inner[index + 1..].iter_mut().for_each(|node| {
                // Update the subsequent siblings by bumping their index by one
                let (path, _) = self.tree.values.get_mut(node.value).expect("every node has a value");
                path[path.len() - 1] += 1;
                let path = path.clone();

                // Update the root of all the children of the preceeding siblings
                node.reparent(&path, &mut self.tree.values);
            });
            value_id
        })?;

        debug_assert_eq!(node_id, self.node_id);
        Some(node_id)
    }
}
