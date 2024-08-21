use super::{NodePath, Tree};

pub trait PathFinder<V> {
    type Output;

    fn apply(&mut self, node: &mut V, path: &NodePath, tree: &mut Tree<V>) -> Self::Output;

    fn parent(&mut self, parent: &V, children: &[u16]);
}
