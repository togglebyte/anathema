use std::ops::{Deref, DerefMut};

pub use self::nodepath::{AsNodePath, new_node_path, root_node};
pub use self::transactions::InsertTransaction;
pub use self::view::TreeView;
use crate::slab::GenSlab;
pub use crate::slab::Key as ValueId;

mod nodepath;
mod transactions;
mod view;

pub type TreeValues<T> = GenSlab<(Box<[u16]>, T)>;

#[derive(Debug)]
pub struct RemovedValues {
    inner: Vec<ValueId>,
}

impl RemovedValues {
    pub const fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn drain(&mut self) -> impl DoubleEndedIterator<Item = ValueId> + '_ {
        self.inner.drain(..)
    }

    pub fn insert(&mut self, value_id: ValueId) {
        self.inner.push(value_id);
    }
}

/// A tree where all values (`T`) are stored in a single contiguous list,
/// and the inner tree (`Nodes`) is made up of branches with indices into
/// the flat list.
///
/// This means fewer allocations when removing entire branches as we can reuse
/// the memory for the values.
#[derive(Debug)]
pub struct Tree<T> {
    layout: Nodes,
    values: TreeValues<T>,
    removed_values: RemovedValues,
}

impl<T> Tree<T> {
    /// Create an empty tree
    pub const fn empty() -> Self {
        Self {
            layout: Nodes::empty(),
            values: TreeValues::empty(),
            removed_values: RemovedValues::new(),
        }
    }

    pub fn view(&mut self) -> TreeView<'_, T> {
        TreeView::new(
            root_node(),
            &mut self.layout,
            &mut self.values,
            &mut self.removed_values,
        )
    }

    /// Get a refernence to a value
    pub fn get_ref(&mut self, value_id: ValueId) -> Option<&T> {
        self.values.get(value_id).map(|(_, value)| value)
    }

    /// Get a mutable refernence to a value
    pub fn get_mut(&mut self, value_id: ValueId) -> Option<&mut T> {
        self.values.get_mut(value_id).map(|(_, value)| value)
    }

    /// Consume the tree and return the values
    pub fn values(self) -> TreeValues<T> {
        self.values
    }

    /// Drain the removed value ids.
    /// This will not return keys that have been replaced.
    pub fn drain_removed(&mut self) -> impl DoubleEndedIterator<Item = ValueId> + '_ {
        self.removed_values.drain()
    }

    /// Perform a given operation (`F`) on a mutable reference to a value in the tree
    /// while still having mutable access to the rest of the tree.
    ///
    /// # Panics
    ///
    /// This will panic if the value is already checked out
    pub fn with_value_mut<F, V>(&mut self, value_id: ValueId, f: F) -> V
    where
        F: FnOnce(&[u16], &mut T, &mut Self) -> V,
    {
        let mut ticket = self.values.checkout(value_id);
        let value = f(&ticket.value.0, &mut ticket.value.1, self);
        self.values.restore(ticket);
        value
    }
}

#[derive(Debug)]
pub struct Nodes {
    pub(crate) inner: Vec<Node>,
}

impl Nodes {
    pub const fn empty() -> Self {
        Self { inner: vec![] }
    }

    /// Find a mutable node by its path
    fn get_by_path_mut(&mut self, mut path: &[u16]) -> Option<&mut Node> {
        let mut nodes = self;
        loop {
            match path {
                [] => break None,
                [i] if (*i as usize) < nodes.len() => break Some(&mut nodes.inner[*i as usize]),
                // The index is outside of the node length
                [_] => break None,
                [i, sub_path @ ..] => {
                    let index = *i as usize;
                    if index >= nodes.len() {
                        break None;
                    }
                    path = sub_path;
                    nodes = &mut nodes.inner[index].children;
                }
            }
        }
    }

    fn insert(&mut self, index: usize, key: ValueId) {
        self.inner.insert(index, Node::new(key));
    }

    fn push(&mut self, key: ValueId) {
        self.inner.push(Node::new(key));
    }

    // Clear nodes and remove associted values
    fn clear<T, F>(&mut self, values: &mut GenSlab<(Box<[u16]>, T)>, removed_values: &mut RemovedValues, f: &mut F)
    where
        F: FnMut(T),
    {
        for mut node in self.inner.drain(..) {
            if let Some((_path, value)) = values.remove(node.value) {
                f(value);
                removed_values.insert(node.value);
            }
            node.children.clear(values, removed_values, f);
        }
    }

    // Unlike the clear function the remove function
    // only remove the node, whereas the values
    // are managed by the three.
    fn remove(&mut self, index: usize) -> Node {
        self.inner.remove(index)
    }

    fn with<'a, F, U: 'a>(&'a self, parent: &[u16], f: F) -> Option<U>
    where
        F: FnOnce(&'a Node) -> U,
    {
        let mut path = parent;
        let mut nodes = self;
        loop {
            match path {
                [] => break None,
                [i] if (*i as usize) < nodes.len() => break Some(f(&nodes.inner[*i as usize])),
                [_] => break None,
                [i, p @ ..] => {
                    let index = *i as usize;
                    if index >= nodes.len() {
                        break None;
                    }
                    path = p;
                    nodes = &nodes.inner[index].children;
                }
            }
        }
    }

    fn with_mut<'a, F, U: 'a>(&'a mut self, parent: &[u16], f: F) -> Option<U>
    where
        F: FnOnce(&'a mut Nodes) -> U,
    {
        let mut path = parent;
        let mut nodes = self;
        loop {
            match path {
                [] => break Some(f(nodes)),
                [i] if (*i as usize) < nodes.len() => break Some(f(&mut nodes.inner[*i as usize].children)),
                // The index is outside of the node length
                [_] => break None,
                [i, sub_path @ ..] => {
                    let index = *i as usize;
                    if index >= nodes.len() {
                        break None;
                    }
                    path = sub_path;
                    nodes = &mut nodes.inner[index].children;
                }
            }
        }
    }
}

impl Deref for Nodes {
    type Target = [Node];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Nodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct Node {
    pub(crate) value: ValueId,
    pub(crate) children: Nodes,
}

impl Node {
    pub fn new(val: ValueId) -> Self {
        Self {
            value: val,
            children: Nodes::empty(),
        }
    }

    pub fn value(&self) -> ValueId {
        self.value
    }

    pub fn children(&self) -> &Nodes {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Nodes {
        &mut self.children
    }

    fn reparent<T>(&mut self, dest: &[u16], values: &mut TreeValues<T>) {
        let (path, _) = values.get_mut(self.value).unwrap();
        path.reparent(dest);
        for child in &mut self.children.inner {
            child.reparent(dest, values);
        }
    }
}
