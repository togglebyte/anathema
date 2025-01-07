use std::fmt::Write;
use std::ops::ControlFlow;

use super::{Node, ValueId};
use crate::slab::GenSlab;

/// Visit each node in the entire tree
pub trait NodeVisitor<T> {
    /// Return control flow.
    /// * `ControlFlow::Continue(())` continue
    /// * `ControlFlow::Break(false)` stop iterating over the children of the current node
    /// * `ControlFlow::Break(true)` stop iterating
    fn visit(&mut self, value: &mut T, path: &[u16], value_id: ValueId) -> ControlFlow<bool>;

    fn push(&mut self) {}

    fn pop(&mut self) {}
}

/// Debug print a tree
pub struct DebugPrintTree {
    output: String,
    level: usize,
}

impl DebugPrintTree {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            level: 0,
        }
    }

    pub fn finish(self) -> String {
        self.output
    }
}

impl<T> NodeVisitor<T> for DebugPrintTree
where
    T: std::fmt::Debug,
{
    fn visit(&mut self, value: &mut T, path: &[u16], _: ValueId) -> ControlFlow<bool> {
        let _ = writeln!(&mut self.output, "{}{path:?}: {value:?}", " ".repeat(self.level * 4),);
        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
    }

    fn pop(&mut self) {
        self.level -= 1;
    }
}

/// Visit each node in the tree
pub fn apply_visitor<T>(
    children: &[Node],
    values: &mut GenSlab<(Box<[u16]>, T)>,
    visitor: &mut impl NodeVisitor<T>,
) -> ControlFlow<bool> {
    for node in children {
        if let Some((path, value)) = values.get_mut(node.value()) {
            if let ControlFlow::Break(stop_propagation) = visitor.visit(value, &*path, node.value()) {
                if stop_propagation {
                    return ControlFlow::Break(true);
                }

                break;
            }

            visitor.push();
            apply_visitor(&node.children, values, visitor)?;
            visitor.pop();
        }
    }

    ControlFlow::Continue(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree::Tree;

    struct Zero;

    impl NodeVisitor<usize> for Zero {
        fn visit(&mut self, value: &mut usize, _path: &[u16], _value_id: ValueId) -> ControlFlow<bool> {
            *value = 0;
            ControlFlow::Continue(())
        }
    }

    #[test]
    fn visit_nodes() {
        // This creates a tree with the following layout:
        // 0 = 42
        //     1 = 42
        //     2 = 42
        //         3 = 42
        //         4 = 42
        //
        // Then we apply the node visitor to the tree
        // This should change all the nodes in the tree to zero.
        //
        // 0 = 0
        //     1 = 0
        //     2 = 0
        //         3 = 0
        //         4 = 0

        let mut tree = Tree::empty();
        let mut tree = tree.view_mut();
        let key = tree.insert(&[]).commit_child(42).unwrap();
        let parent = tree.path(key);
        let _ = tree.insert(&parent).commit_child(42).unwrap();
        let key = tree.insert(&parent).commit_child(42).unwrap();
        let parent = tree.path(key);
        tree.insert(&parent).commit_child(42).unwrap();
        tree.insert(&parent).commit_child(42).unwrap();

        tree.apply_visitor(&mut Zero);

        let values = tree
            .values
            .iter()
            .map(|(_path, value)| value)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(values, vec![0, 0, 0, 0, 0]);
    }
}
