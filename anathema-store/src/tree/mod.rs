use std::ops::{Deref, DerefMut};

pub use self::foreach::Generator;
pub use self::iter::{TreeFilter, TreeForEach};
pub use self::nodepath::{new_node_path, root_node, AsNodePath};
pub use self::pathfinder::PathFinder;
pub use self::transactions::InsertTransaction;
pub use self::view::TreeView;
use self::visitor::NodeVisitor;
use crate::slab::GenSlab;
pub use crate::slab::Key as ValueId;

mod foreach;
mod iter;
mod nodepath;
mod pathfinder;
mod transactions;
mod view;
pub mod visitor;

pub type TreeValues<T> = GenSlab<(Box<[u16]>, T)>;

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
    removed_values: Vec<ValueId>,
}

impl<T> Tree<T> {
    /// Create an empty tree
    pub const fn empty() -> Self {
        Self {
            layout: Nodes::empty(),
            values: TreeValues::empty(),
            removed_values: Vec::new(),
        }
    }

    pub fn view_mut(&mut self) -> TreeView<'_, T> {
        TreeView::new(
            root_node(),
            &mut self.layout,
            &mut self.values,
            &mut self.removed_values,
        )
    }

    pub fn get_ref(&mut self, value_id: ValueId) -> Option<&T> {
        self.values.get(value_id).map(|(_, value)| value)
    }

    pub fn get_mut(&mut self, value_id: ValueId) -> Option<&mut T> {
        self.values.get_mut(value_id).map(|(_, value)| value)
    }

    pub fn values(self) -> TreeValues<T> {
        self.values
    }

    // TODO: remove this gross function
    pub fn value_len(&self) -> usize {
        self.values.iter().count()
    }

    /// Split the tree into values and structure
    pub fn split(&mut self) -> (&Nodes, &mut TreeValues<T>) {
        (&self.layout, &mut self.values)
    }

    /// Give a capacity to the underlying value store.
    /// This will not affect the storage of the layout.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            layout: Nodes::empty(),
            values: TreeValues::with_capacity(cap),
            removed_values: Vec::new(),
        }
    }

    /// The root node
    pub fn root(&self) -> &Node {
        &self.layout[0]
    }

    /// Mutable iterator over node paths and values
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (Box<[u16]>, T)> {
        self.values.iter_mut()
    }

    /// Drain the removed value ids.
    /// This will not return keys that have been replaced.
    pub fn drain_removed(&mut self) -> impl DoubleEndedIterator<Item = ValueId> + '_ {
        self.removed_values.drain(..)
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

    /// Try to get a path for a given value id.
    /// There is no guarantee that the path exists for this key.
    pub fn try_path(&self, id: impl Into<ValueId>) -> Option<Box<[u16]>> {
        self.try_path_ref(id).map(Into::into)
    }

    /// Try to get a path for a given value id.
    /// There is no guarantee that the path exists for this key.
    pub fn try_path_ref(&self, id: impl Into<ValueId>) -> Option<&[u16]> {
        let id = id.into();
        let (path, _) = self.values.get(id)?;
        Some(path)
    }

    /// Get a reference to a `Node` via a path.
    pub fn get_node_by_path(&mut self, path: &[u16]) -> Option<(&Node, &mut TreeValues<T>)> {
        self.layout.with(path, |node| node).map(|node| (node, &mut self.values))
    }

    /// Remove a `Node` and value from the tree.
    /// This will also remove all the children and associated values.
    pub fn remove(&mut self, path: &[u16]) {
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
            let _ = self
                .values
                .remove(value_key)
                .expect("a node is always associated with a value");
            node.children.clear(&mut self.values, &mut self.removed_values);
        }
    }

    /// Remove the children of a `Node`. This
    /// will also remove all the associated values.
    pub fn remove_children(&mut self, path: &[u16]) {
        let Some((path, index)) = path.split_parent() else { return };
        let Some(Some(node)) = self.layout.with_mut(path, |nodes| nodes.get_mut(index)) else { return };
        node.children.clear(&mut self.values, &mut self.removed_values);
    }

    pub fn for_each<'filter, F: TreeFilter>(&mut self, filter: &'filter mut F) -> TreeForEach<'_, 'filter, T, F> {
        TreeForEach {
            nodes: &self.layout,
            values: &mut self.values,
            filter,
        }
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

    /// Get mutable access to a node value along with the children
    /// of that value, while still having mutable access to the values.
    pub fn with_nodes_and_values<F>(&mut self, value_id: ValueId, mut f: F)
    where
        F: FnMut(&mut T, &[Node], &mut TreeValues<T>),
    {
        let mut ticket = self.values.checkout(value_id);
        let node = self
            .layout
            .get_by_path(&ticket.value.0)
            .expect("the value and the node exists at the same time");
        f(&mut ticket.value.1, node.children(), &mut self.values);
        self.values.restore(ticket);
    }

    /// Apply function to each child of a parent path.
    #[deprecated]
    pub fn children_of<F>(&mut self, parent: &[u16], mut f: F) -> Option<()>
    where
        F: FnMut(&Node, &mut TreeValues<T>),
    {
        // Special case if the `parent` is the tree itself.
        if parent.is_empty() {
            self.layout.iter().for_each(|node| f(node, &mut self.values));
            return Some(());
        }

        self.layout.with(parent, |node| {
            node.children().iter().for_each(|node| f(node, &mut self.values));
        });
        Some(())
    }

    /// Apply function to each sibling after the given path.
    pub fn children_after<F>(&mut self, path: &[u16], mut f: F) -> Option<()>
    where
        F: FnMut(&Node, &mut TreeValues<T>),
    {
        let (parent, index) = path.split_parent()?;
        self.layout.with(parent, |parent| {
            parent
                .children()
                .iter()
                .skip(index + 1)
                .for_each(|node| f(node, &mut self.values));
        });
        Some(())
    }

    /// Apply the [`PathFinder`] to a given path.
    pub fn apply_path_finder<P>(&mut self, node_path: &[u16], path_finder: P)
    where
        P: PathFinder<Input = T>,
    {
        apply_path_finder(self, node_path, path_finder);
    }

    // /// Apply the [`NodeWalker`].
    // pub fn apply_node_walker(&mut self, path: &[u16], mut walker: impl NodeWalker) {
    //     unimplemented!()
    //     // walker::walk_the_walker(&self.layout, &mut self.values, path, &mut walker)
    // }

    /// Apply a [`NodeVisitor`], depth first
    pub fn apply_visitor<V: NodeVisitor<T>>(&mut self, visitor: &mut V) {
        visitor::apply_visitor(&self.layout, &mut self.values, visitor);
    }

    /// Split the tree giving access to the layout and the values.
    pub fn split_mut(&mut self) -> (&[Node], &mut TreeValues<T>) {
        (&self.layout, &mut self.values)
    }

    pub fn is_vacant(&self, key: ValueId) -> bool {
        self.values.is_vacant(key)
    }
}

fn apply_path_finder<P>(tree: &mut Tree<P::Input>, node_path: &[u16], mut path_finder: P)
where
    P: PathFinder,
{
    let mut path: &[u16] = node_path;
    let mut nodes: &[_] = &tree.layout.inner;
    let values = &mut tree.values;

    loop {
        match path {
            [] => break,
            [i] => {
                // Found the node
                let node = &nodes[*i as usize];
                tree.with_value_mut(node.value(), |path, widget, tree| {
                    path_finder.apply(widget, path, tree);
                });
                break;
            }
            [i, sub_path @ ..] => {
                let index = *i as usize;
                if index >= nodes.len() {
                    break;
                }
                path = sub_path;
                let node = &nodes[index];

                let node_id = node.value();

                let parent = values
                    .get_mut(node_id)
                    .map(|(_, val)| val)
                    .expect("a node always has a matching value");

                path_finder.parent(parent, sub_path);
                nodes = node.children();
            }
        }
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

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.inner.iter()
    }

    pub fn iter_with_values<'a, T>(
        &'a self,
        values: &'a TreeValues<T>,
    ) -> impl Iterator<Item = (&'a Node, &'a Box<[u16]>, &'a T)> {
        self.inner.iter().filter_map(|node| {
            let (path, value) = values.get(node.value)?;
            Some((node, path, value))
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Find a node by its path
    fn get_by_path(&self, mut path: &[u16]) -> Option<&Node> {
        let mut nodes = self;
        loop {
            match path {
                [] => break None,
                [i] if (*i as usize) < nodes.len() => break Some(&nodes.inner[*i as usize]),
                // The index is outside of the node length
                [_] => break None,
                [i, sub_path @ ..] => {
                    let index = *i as usize;
                    if index >= nodes.len() {
                        break None;
                    }
                    path = sub_path;
                    nodes = &nodes.inner[index].children;
                }
            }
        }
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

    fn get_mut(&mut self, index: usize) -> Option<&mut Node> {
        self.inner.get_mut(index)
    }

    fn insert(&mut self, index: usize, key: ValueId) {
        self.inner.insert(index, Node::new(key));
    }

    fn push(&mut self, key: ValueId) {
        self.inner.push(Node::new(key));
    }

    // Clear nodes and remove associted values
    fn clear<T>(&mut self, values: &mut GenSlab<(Box<[u16]>, T)>, removed_values: &mut Vec<ValueId>) {
        for mut node in self.inner.drain(..) {
            let _ = values.remove(node.value);
            removed_values.push(node.value);
            node.children.clear(values, removed_values);
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

    pub fn children_mut(&mut self) -> &mut Nodes {
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

#[cfg(test)]
mod test {
    use super::*;
}
