use super::Node;
use crate::slab::GenSlab;

/// Call the apply function for every value in the tree that is part of the path,
/// and then on all the children (recursively) for the selected node.
pub trait NodeWalker<V> {
    fn apply(&mut self, node: &mut V);
}

/// This function will apply the `walker` to all the nodes in the path.
/// Once it finds the target node, it will apply the walker to the node
/// and all the children and childrens children etc. of that node.
pub fn walk_the_walker<T>(
    mut nodes: &[Node],
    values: &mut GenSlab<(Box<[u16]>, T)>,
    mut path: &[u16],
    walker: &mut impl NodeWalker<T>,
) {
    loop {
        match path {
            [] => break,
            [i] => {
                // Found the node
                let index = *i as usize;
                let node = &nodes[index];
                let node_id = node.value();

                let value = values
                    .get_mut(node_id)
                    .map(|(_, val)| val)
                    .expect("a node always has a matching value");

                walker.apply(value);

                // Apply the walker to all the children
                apply_walker(node.children(), values, walker);
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

                walker.apply(parent);
                nodes = node.children();
            }
        }
    }
}

/// This function will apply the walker to all the children
fn apply_walker<T>(children: &[Node], values: &mut GenSlab<(Box<[u16]>, T)>, walker: &mut impl NodeWalker<T>) {
    for child in children {
        let node_id = child.value();
        let value = values
            .get_mut(node_id)
            .map(|(_, val)| val)
            .expect("a node always has a matching value");
        walker.apply(value);
        apply_walker(child.children(), values, walker);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree::Tree;

    struct Zero;

    impl NodeWalker<usize> for Zero {
        fn apply(&mut self, node: &mut usize) {
            *node = 0;
        }
    }

    #[test]
    fn walk_the_tree() {
        // This creates a tree with the following layout:
        // 0 = 42
        //     1 = 42
        //     2 = 42
        //         3 = 42
        //         4 = 42
        //
        // Then we apply the node walker to the node with an id of 2
        // This should change all the nodes on the way to 2, and all the children of 2.
        //
        // 0 = 0
        //     1 = 42
        //     2 = 0
        //         3 = 0
        //         4 = 0

        let mut tree = Tree::empty();
        let key = tree.insert(&[]).commit_child(42).unwrap();
        let parent = tree.path(key);
        let _ = tree.insert(&parent).commit_child(42).unwrap();
        let key = tree.insert(&parent).commit_child(42).unwrap();
        let parent = tree.path(key);
        tree.insert(&parent).commit_child(42).unwrap();
        tree.insert(&parent).commit_child(42).unwrap();

        let (nodes, values) = tree.split();
        walk_the_walker(nodes, values, &parent, &mut Zero);

        let values = values.iter().map(|(_path, value)| value).copied().collect::<Vec<_>>();
        assert_eq!(values, vec![0, 42, 0, 0, 0]);
    }
}
