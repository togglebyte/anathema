use super::TreeView;

/// The generator should generate child nodes, created from a parent node.
pub trait Generator<T, C> {
    type Output;

    fn from_value(value: &mut T, ctx: &mut C) -> Self
    where
        Self: Sized;

    fn generate(&mut self, tree: &mut TreeView<'_, T>, ctx: &mut C) -> Self::Output;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::slab::Key;
    use crate::tree::{root_node, Nodes, Tree, TreeValues};

    type Size = usize;

    struct Ctx<'frame> {
        scopes: &'frame mut Vec<()>,
    }

    impl Ctx<'_> {
        fn scope(&mut self) {
            eprintln!("Scoping...");
        }
    }

    #[derive(Debug)]
    enum Blueprint {
        Single(Single),
        Loop { counter: usize, body: Vec<Blueprint> },
    }

    impl Blueprint {
        fn children(&self) -> &[Blueprint] {
            match self {
                Blueprint::Single(single) => &single.children,
                Blueprint::Loop { body, .. } => body,
            }
        }
    }

    #[derive(Debug)]
    struct Single {
        value: usize,
        children: Vec<Blueprint>,
    }

    enum Value<'bp> {
        Single(usize, &'bp [Blueprint]),
        Loop(usize, &'bp [Blueprint]),
        Iter(&'bp [Blueprint]),
    }

    impl std::fmt::Debug for Value<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Single(val, _) => write!(f, "Single: {val}"),
                Value::Loop(count, body) => write!(f, "Loop {count}"),
                Value::Iter(_) => write!(f, "Iter"),
            }
        }
    }

    impl<'bp> Value<'bp> {
        fn layout(&self, ctx: &mut Ctx<'bp>, mut children: ForEach2<'_, Value<'bp>, Tr, Gen<'bp>, Ctx<'bp>>) -> Size {
            let mut size = 1;
            let mut skipper = 2usize;
            children.each(ctx, |ctx, value, children| {
                eprintln!("{value:?}");
                // eprintln!("starting layout for {value:?}");
                let new_size = value.layout(ctx, children);
                skipper = skipper.saturating_sub(new_size);
                if skipper > 0 {
                    return ControlFlow::Continue(());
                }

                // eprintln!("ending layout for {value:?}");

                if size > 2 {
                    return ControlFlow::Break(());
                }

                ControlFlow::Continue(())
            });
            size
        }
    }

    fn eval<'bp>(blueprint: &'bp Blueprint, tree: &mut TreeView<'_, Value<'bp>>) -> Key {
        match blueprint {
            Blueprint::Single(single) => {
                let value = Value::Single(single.value, &single.children);
                tree.insert(tree.offset).commit_child(value).unwrap()
            }
            Blueprint::Loop { counter, body } => {
                let value = Value::Loop(*counter, body);
                tree.insert(tree.offset).commit_child(value).unwrap()
            }
        }
    }

    struct Tr;

    impl<'bp> Traverser<Value<'bp>> for Tr {
        fn traverse(&self, input: &mut Value<'bp>) -> bool {
            match input {
                Value::Single(_, _) => false,
                Value::Loop { .. } => true,
                Value::Iter(_) => true,
            }
        }
    }

    enum Gen<'bp> {
        Loop(usize, &'bp [Blueprint]),
        Iter(&'bp [Blueprint]),
        Single(&'bp [Blueprint]),
        Empty,
    }

    // struct Gen<'a, 'bp> {
    //     value: &'a Value<'bp>
    // }

    impl<'frame, 'bp> Generator<Value<'bp>, Ctx<'frame>> for Gen<'bp> {
        fn generate(&mut self, tree: &mut TreeView<'_, Value<'bp>>, ctx: &mut Ctx<'frame>) -> bool {
            match self {
                Gen::Loop(counter, body) if *counter > 0 && tree.layout.inner.len() != *counter => {
                    *counter -= 1;
                    eprintln!("counter: {counter}");
                    let value = Value::Iter(body);
                    tree.insert(tree.offset).commit_child(value);
                }
                Gen::Loop(counter, body) => return false,
                Gen::Single(blueprints) | Gen::Iter(blueprints) => {
                    if blueprints.is_empty() {
                        return false;
                    }

                    let index = tree.layout.inner.len();
                    if index == blueprints.len() {
                        return false;
                    }

                    let blueprint = &blueprints[index];
                    eval(blueprint, tree);
                }
                Gen::Empty => return false,
            }
            true
        }

        fn from_value(value: &mut Value<'bp>, _ctx: &mut Ctx<'frame>) -> Self
        where
            Self: Sized,
        {
            match value {
                Value::Single(_, children) => Self::Single(children),
                Value::Loop(counter, body) => Self::Loop(*counter, body),
                Value::Iter(body) => Self::Iter(body),
            }
        }
    }

    fn print_tree(nodes: &Nodes, values: &TreeValues<Value<'_>>, indent: usize) {
        let space = " ".repeat(indent * 4);
        for node in nodes.iter() {
            let value = values.get(node.value).unwrap();
            eprintln!("{space}{value:?}");
            print_tree(&node.children, values, indent + 1);
        }
    }
}
