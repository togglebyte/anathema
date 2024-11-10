use super::{InsertTransaction, Nodes, TreeValues, ValueId};

#[derive(Debug)]
pub struct TreeView<'tree, T> {
    pub(super) offset: &'tree [u16],
    pub(super) values: &'tree mut TreeValues<T>,
    pub(super) layout: &'tree mut Nodes,
}

impl<'tree, T> TreeView<'tree, T> {
    pub(super) fn new(offset: &'tree [u16], layout: &'tree mut Nodes, values: &'tree mut TreeValues<T>) -> Self {
        Self { offset, values, layout }
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&[u16], &mut T, TreeView<'_, T>),
    {
        for index in 0..self.layout.len() {
            let node = &mut self.layout.inner[index];
            self.values.with_mut(node.value, |(offset, value), values| {
                let tree_view = TreeView::new(offset, &mut node.children, values);
                f(offset, value, tree_view);
            });
        }
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

    /// Being an insert transaction.
    /// The transaction has to be committed before the value is written to
    /// the tree.
    /// ```
    /// # use anathema_store::tree::*;
    /// let mut tree = Tree::empty();
    /// let transaction = tree.insert(&[]);
    /// let value_id = transaction.commit_child(1usize).unwrap();
    /// let one = tree.get_ref_by_id(value_id).unwrap();
    /// assert_eq!(*one, 1);
    /// ```
    pub fn insert<'a>(&'a mut self, parent: &'a [u16]) -> InsertTransaction<'a, 'tree, T> {
        InsertTransaction::new(self, parent)
    }

    /// Perform a given operation (`F`) on a mutable reference to a value in the tree
    /// while still having mutable access to the rest of the tree.
    ///
    /// # Panics
    ///
    /// This will panic if the value is already checked out
    pub fn with_value_mut<F, V>(&mut self, value_id: ValueId, f: F) -> V
    where
        F: FnOnce(&[u16], &mut T, TreeView<'_, T>) -> V,
    {
        let mut ticket = self.values.checkout(value_id);
        let view = TreeView {
            offset: self.offset,
            values: self.values,
            layout: self.layout,
        };
        let value = f(&ticket.value.0, &mut ticket.value.1, view);
        self.values.restore(ticket);
        value
    }
}
