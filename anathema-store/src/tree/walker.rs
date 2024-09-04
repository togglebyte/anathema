/// Call the apply function for every value in the tree that is part of the path.
pub trait NodeWalker<V> {
    fn apply(&mut self, node: &mut V);
}
