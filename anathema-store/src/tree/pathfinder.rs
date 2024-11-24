use super::Tree;

pub trait PathFinder {
    type Input;

    type Output;

    fn apply(&mut self, node: &mut Self::Input, path: &[u16], tree: &mut Tree<Self::Input>) -> Self::Output;

    fn parent(&mut self, parent: &mut Self::Input, sub_path: &[u16]);
}
