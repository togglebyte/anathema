use super::{NodePath, Tree};

pub trait PathFinder<V> {
    fn apply(&mut self, node: &mut V, path: &NodePath, tree: &mut Tree<V>);

    fn parent(&mut self, parent: &V, children: &[u16]);
}
