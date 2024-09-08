use super::Tree;

pub trait PathFinder<V> {
    type Output;

    fn apply(&mut self, node: &mut V, path: &[u16], tree: &mut Tree<V>) -> Self::Output;

    fn parent(&mut self, parent: &mut V, children: &[u16]);
}
