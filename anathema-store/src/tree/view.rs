use std::ops::ControlFlow;

use super::{ForEach2, Generator, InsertTransaction, Nodes, Traverser, TreeValues, ValueId};

#[derive(Debug)]
pub struct TreeView<'tree, T> {
    pub offset: &'tree [u16],
    pub values: &'tree mut TreeValues<T>,
    pub layout: &'tree mut Nodes,
}

impl<'tree, T> TreeView<'tree, T> {
    pub fn new(offset: &'tree [u16], layout: &'tree mut Nodes, values: &'tree mut TreeValues<T>) -> Self {
        Self { offset, values, layout }
    }

    pub fn layout_len(&self) -> usize {
        self.layout.len()
    }

    #[deprecated]
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

    pub fn each<C, F, Tr, Gen>(&mut self, ctx: &mut C, traverser: &Tr, mut f: F)
    where
        Gen: Generator<T, C>,
        Tr: Traverser<T>,
        for<'a> F: FnMut(&mut C, &mut T, ForEach2<'a, T, Tr, Gen, C>) -> ControlFlow<()>,
    {
        for node in &mut self.layout.inner {
            self.values.with_mut(node.value, |(path, value), values| {
                let view = TreeView::new(path, &mut node.children, values);
                let gen = Gen::from_value(value, ctx);
                let children = ForEach2::new(view, traverser, gen);
                f(ctx, value, children);
            })
        }
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
        let (path, value) = &mut *ticket;
        let node = self.layout.get_by_path_mut(&path[self.offset.len()..]).unwrap(); // TODO: unwrap decide what to do with this horrid unwrap
        let view = TreeView {
            offset: path,
            values: self.values,
            layout: node.children_mut(),
        };
        let value = f(path, value, view);
        self.values.restore(ticket);
        value
    }
}
