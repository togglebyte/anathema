use std::ops::ControlFlow;

use super::visitor::NodeVisitor;
use super::{AsNodePath, InsertTransaction, Nodes, TreeValues, ValueId};

#[derive(Debug)]
pub struct TreeView<'tree, T> {
    pub offset: &'tree [u16],
    pub values: &'tree mut TreeValues<T>,
    pub layout: &'tree mut Nodes,
    pub removed_values: &'tree mut Vec<ValueId>,
}

impl<'tree, T> TreeView<'tree, T> {
    pub fn new(
        offset: &'tree [u16],
        layout: &'tree mut Nodes,
        values: &'tree mut TreeValues<T>,
        removed_values: &'tree mut Vec<ValueId>,
    ) -> Self {
        Self {
            offset,
            values,
            layout,
            removed_values,
        }
    }

    pub fn view_mut(&mut self) -> TreeView<'_, T> {
        TreeView::new(
            self.offset,
            &mut self.layout,
            &mut self.values,
            &mut self.removed_values,
        )
    }

    pub fn get_mut(&mut self, value_id: ValueId) -> Option<&mut T> {
        self.values.get_mut(value_id).map(|(_, value)| value)
    }

    pub fn contains(&self, key: ValueId) -> bool {
        self.values.get(key).is_some()
    }

    /// The number of children (not counting childrens children)
    pub fn layout_len(&self) -> usize {
        self.layout.len()
    }

    pub fn for_each<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&[u16], &mut T, TreeView<'_, T>) -> ControlFlow<U>,
    {
        for index in 0..self.layout.len() {
            let node = &mut self.layout.inner[index];
            match self.values.with_mut(node.value, |(offset, value), values| {
                let tree_view = TreeView::new(offset, &mut node.children, values, self.removed_values);
                f(offset, value, tree_view)
            }) {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(value) => return Some(value),
            }
        }

        None
    }

    /// The path reference for a value in the tree.
    /// Unlike a `ValueId` which will never change for a given value,
    /// the `NodePath` can change if the node is moved to another location within the tree.
    ///
    /// # Panics
    ///
    /// Panics if the value id is no long present in the tree.
    pub fn path_ref(&self, id: impl Into<ValueId>) -> &[u16] {
        let id = id.into();
        let (path, _) = self
            .values
            .get(id)
            .expect("an id should always be associated with a path");
        path
    }

    /// The path to a value in the tree.
    /// Unlike a `ValueId` which will never change for a given value,
    /// the `NodePath` can change if the node is moved to another location within the tree.
    ///
    /// # Panics
    ///
    /// Panics if the value id is no long present in the tree.
    pub fn path(&self, id: impl Into<ValueId>) -> Box<[u16]> {
        self.path_ref(id).into()
    }

    /// Find the value id by the path
    pub fn id(&self, path: &[u16]) -> Option<ValueId> {
        self.layout.with(path, |nodes| nodes.value())
    }

    /// Get a mutable reference by value id
    pub fn get_mut_by_id(&mut self, node_id: ValueId) -> Option<&mut T> {
        self.values.get_mut(node_id).map(|(_, val)| val)
    }

    /// Get a mutable reference by path.
    /// This has an additional cost since the value id has to
    /// be found first.
    pub fn get_mut_by_path(&mut self, path: &[u16]) -> Option<&mut T> {
        let relative = &path[self.offset.len()..];
        let id = self.id(relative)?;
        self.values.get_mut(id).map(|(_, val)| val)
    }

    /// Get a reference by value id
    pub fn get_ref_by_id(&self, node_id: ValueId) -> Option<&T> {
        self.values.get(node_id).map(|(_, val)| val)
    }

    /// Get a reference by path.
    /// This has an additional cost since the value id has to
    /// be found first.
    pub fn get_ref_by_path(&self, path: &[u16]) -> Option<&T> {
        let id = self.id(path)?;
        self.values.get(id).map(|(_, val)| val)
    }

    /// Get a reference to the value and the value id
    /// This has an additional cost since the value id has to
    /// be found first.
    pub fn get_node_and_value(&self, path: &[u16]) -> Option<(ValueId, &T)> {
        let id = self.id(path)?;
        self.values.get(id).map(|(_, val)| (id, val))
    }

    /// Being an insert transaction.
    /// The transaction has to be committed before the value is written to
    /// the tree.
    /// ```
    /// # use anathema_store::tree::*;
    /// let mut tree = Tree::empty();
    /// let mut tree = tree.view_mut();
    /// let transaction = tree.insert(&[]);
    /// let value_id = transaction.commit_child(1usize).unwrap();
    /// let one = tree.get_ref_by_id(value_id).unwrap();
    /// assert_eq!(*one, 1);
    /// ```
    pub fn insert<'a>(&'a mut self, parent: &'a [u16]) -> InsertTransaction<'a, 'tree, T> {
        InsertTransaction::new(self, parent)
    }

    /// Remove all the nodes in this view
    pub fn truncate_children(&mut self) {
        self.layout.clear(&mut self.values, &mut self.removed_values);
    }

    /// Remove a `Node` and value from the tree.
    /// This will also remove all the children and associated values.
    pub fn relative_remove(&mut self, path: &[u16]) {
        if self.layout.is_empty() {
            return;
        }

        // This will not return the value that was removed, as it will also
        // remove all the children under that node.
        let (path, index) = path.split_parent().expect("a value will always exist within the tree");

        let node = self.layout.with_mut(path, |nodes| {
            let node = nodes.remove(index);
            self.removed_values.push(node.value);

            nodes.inner[index..].iter_mut().for_each(|node| {
                // Update the subsequent siblings by bumping their index by one
                let (path, _) = self.values.get_mut(node.value).expect("every node has a value");
                path[path.len() - 1] -= 1;

                // Clone the path to drop the borrow of the tree
                let path = path.clone();
                // Update the root of all the children of the preceeding siblings
                node.reparent(&path, &mut self.values);
            });

            node
        });

        if let Some(mut node) = node {
            let value_key = node.value();
            let _value = self
                .values
                .remove(value_key)
                .expect("a node is always associated with a value");
            node.children.clear(&mut self.values, &mut self.removed_values);
        }
    }

    /// Perform a given operation (`F`) on a mutable reference to a value in the tree
    /// while still having mutable access to the rest of the tree.
    ///
    /// # Panics
    ///
    /// This will panic if the value is already checked out
    pub fn with_value_mut<F, V>(&mut self, value_id: ValueId, f: F) -> Option<V>
    where
        F: FnOnce(&[u16], &mut T, TreeView<'_, T>) -> V,
    {
        let mut ticket = self.values.checkout(value_id);
        let (path, value) = &mut *ticket;
        let node = self.layout.get_by_path_mut(&path[self.offset.len()..])?;
        let view = TreeView {
            offset: path,
            values: self.values,
            layout: node.children_mut(),
            removed_values: self.removed_values,
        };
        let value = f(path, value, view);
        self.values.restore(ticket);
        Some(value)
    }

    /// Apply a [`NodeVisitor`], depth first
    pub fn apply_visitor<V: NodeVisitor<T>>(&mut self, visitor: &mut V) {
        super::visitor::apply_visitor(&self.layout, &mut self.values, visitor);
    }

    /// Perform a given operation (`F`) on a reference to a value in the tree
    /// while still haveing mutable access to the rest of the tree.
    pub fn with_value<F, R>(&self, value_id: ValueId, mut f: F) -> Option<R>
    where
        F: FnMut(&[u16], &T, &Self) -> R,
    {
        let value = self.values.get(value_id)?;
        let ret = f(&value.0, &value.1, self);
        Some(ret)
    }

    pub fn each_value<F>(&mut self, f: F)
    where
        F: FnMut(&mut T),
    {
        self.values.iter_mut().map(|(_, val)| val).for_each(f);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree::{root_node, Tree};

    #[derive(Debug, PartialEq)]
    enum Value {
        S(String),
        I(usize),
    }

    #[test]
    fn insert_and_commit() {
        let mut tree = Tree::<u32>::empty();
        let mut tree = tree.view_mut();
        let transaction = tree.insert(root_node());
        let node_id = transaction.node_id();
        let value = 123;

        transaction.commit_child(value);

        assert_eq!(*tree.get_ref_by_id(node_id).unwrap(), 123);
    }

    #[test]
    fn insert_without_commit() {
        let mut tree = Tree::<()>::empty();
        let mut tree = tree.view_mut();
        let transaction = tree.insert(root_node());
        let node_id = transaction.node_id();
        assert!(tree.get_ref_by_id(node_id).is_none());
    }

    #[test]
    fn get_by_path() {
        let mut tree = Tree::empty();
        let mut tree = tree.view_mut();
        let node_id = tree.insert(root_node()).commit_child(1).unwrap();
        let path = tree.path(node_id);
        tree.insert(&path).commit_child(2);

        let one = tree.get_ref_by_path(&[0]).unwrap();
        let two = tree.get_ref_by_path(&[0, 0]).unwrap();

        assert_eq!(*one, 1);
        assert_eq!(*two, 2);
    }

    #[test]
    fn with_node_id() {
        let mut tree = Tree::empty();
        let mut tree = tree.view_mut();
        let key = tree.insert(root_node()).commit_child(0).unwrap();
        tree.insert(root_node()).commit_child(1);
        tree.with_value(key, |_path, _value, _tree| {});
    }

    #[test]
    fn with_node_id_reading_checkedout_value() {
        let mut tree = Tree::empty();
        let mut tree = tree.view_mut();
        let key = tree.insert(root_node()).commit_child(0).unwrap();
        tree.insert(root_node()).commit_child(1);
        tree.with_value_mut(key, |_path, _value, tree| {
            // The value is already checked out
            assert!(tree.get_ref_by_id(key).is_none());
        });
    }

    #[test]
    fn remove_children() {
        let mut tree = Tree::<u32>::empty();
        let mut tree = tree.view_mut();
        tree.insert(root_node()).commit_child(1);
        let path = &[0, 0];
        tree.insert(path).commit_at(2);

        assert!(tree.get_ref_by_path(path).is_some());
        tree.relative_remove(path);
        assert!(tree.get_ref_by_path(path).is_none());
    }

    // This is where we start:
    // Insert At has to be a posibility.
    // Scenario:
    // * Insert at 0
    // * Insert at len
    // * Insert in the middle
    #[test]
    fn insert_at_path() {
        let mut tree = Tree::empty();
        let mut tree = tree.view_mut();
        // Setup: add two entries

        // First entry
        let key = tree.insert(root_node()).commit_child(0).unwrap();
        let _sibling_path = tree.path_ref(key);

        // Second entry (with two children)
        let key_1 = tree.insert(root_node()).commit_child(1).unwrap();
        let parent: Box<_> = tree.path_ref(key_1).into();

        // Insert two values under the second entry
        let key_1_0 = tree.insert(&parent).commit_child(5).unwrap();
        let key_1_1 = tree.insert(&parent).commit_child(6).unwrap();

        // Assert 1.
        // First assertion that the paths are all rooted in [1]
        assert_eq!(tree.path_ref(key_1), &[1]);
        assert_eq!(tree.path_ref(key_1_0), &[1, 0]);
        assert_eq!(tree.path_ref(key_1_1), &[1, 1]);

        // Insert a node as the new first node ([0]), which should update
        // the path for all the other entries in the tree
        let insert_at = &[0];
        tree.insert(insert_at).commit_at(123).unwrap();

        // Assert 2
        // Second assertion that the paths are all rooted in [2]
        assert_eq!(tree.path_ref(key_1), &[2]);
        assert_eq!(tree.path_ref(key_1_0), &[2, 0]);
        assert_eq!(tree.path_ref(key_1_1), &[2, 1]);

        // Insert a node as the new last node ([0]), which should update
        // the path for all the other entries in the tree
        let insert_at = [3];
        let key_3 = tree.insert(&insert_at).commit_at(999).unwrap();

        // Assert 3
        assert_eq!(tree.path_ref(key_3), &[3]);
    }

    #[test]
    fn modify_tree() {
        let mut tree = Tree::<Value>::empty();
        tree.view_mut().insert(root_node()).commit_child(Value::I(123));
        let path = &[0, 0];
        tree.view_mut().insert(path).commit_at(Value::I(1));

        let mut tree = tree.view_mut();
        tree.for_each(|_path, outer_value, mut children| {
            children.for_each(|_path, inner_value, mut children| {
                let parent = &[0, 0];
                children.insert(parent).commit_child(Value::I(999));
                let path = &[0, 0, 0];
                let value = children.get_mut_by_path(path).unwrap();
                assert_eq!(*value, Value::I(999));
                assert_eq!(*outer_value, Value::I(123));
                assert_eq!(*inner_value, Value::I(1));
                ControlFlow::Continue::<(), _>(())
            });
            ControlFlow::Break(())
        });
    }
}
