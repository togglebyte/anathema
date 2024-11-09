use super::{InsertTransaction, Nodes, TreeValues, ValueId};

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
        F: FnMut(&mut T, TreeView<'_, T>),
    {
        for index in 0..self.layout.len() {
            let node = &mut self.layout.inner[index];
            self.values.with_mut(node.value, |(offset, value), values| {
                let tree_view = TreeView::new(offset, &mut node.children, values);
                f(value, tree_view);
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
}

