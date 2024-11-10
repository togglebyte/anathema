use std::ops::ControlFlow;

use super::TreeView;

// NOTE
// * A generator can not have a blueprint since it can be an `Iter` of a `Loop`
// * It can have a body?
// * Loop -> Iter -> &[Blueprint]
//
// A blueprint becomes a widget and gets a generator?

struct ForEach2<'tree, V, T, G, C> {
    tree: TreeView<'tree, V>,
    traverser: &'tree T,
    generator: G,
    _p: std::marker::PhantomData<C>,
}

impl<'tree, 'bp, V, T, G, C> ForEach2<'tree, V, T, G, C>
where
    G: Generator<V, C>,
    T: Traverser<V>,
    G: Generator<V, C>,
{
    pub fn new(tree: TreeView<'tree, V>, traverser: &'tree T, generator: G) -> Self {
        Self {
            tree,
            traverser,
            generator,
            _p: Default::default(),
        }
    }

    pub fn each<F>(&mut self, ctx: &mut C, mut f: F) -> ControlFlow<()>
    where
        for<'a> F: FnMut(&mut C, &mut V, ForEach2<'a, V, T, G, C>) -> ControlFlow<()>,
    {
        self.inner_each(ctx, &mut f)
    }

    fn inner_each<F>(&mut self, ctx: &mut C, f: &mut F) -> ControlFlow<()>
    where
        for<'a> F: FnMut(&mut C, &mut V, ForEach2<'a, V, T, G, C>) -> ControlFlow<()>,
    {
        for index in 0..self.tree.layout.len() {
            self.process(index, ctx, f)?;
        }

        // -----------------------------------------------------------------------------
        //   - Generation of values -
        // -----------------------------------------------------------------------------
        loop {
            let index = self.tree.layout.len();
            if !self.generator.generate(&mut self.tree, ctx) {
                return ControlFlow::Continue(());
            }
            self.process(index, ctx, f)?;
        }
    }

    fn process<F>(&mut self, index: usize, ctx: &mut C, f: &mut F) -> ControlFlow<()>
    where
        T: Traverser<V>,
        G: Generator<V, C>,
        for<'a> F: FnMut(&mut C, &mut V, ForEach2<'a, V, T, G, C>) -> ControlFlow<()>,
    {
        let Some(node) = self.tree.layout.inner.get_mut(index) else { return ControlFlow::Continue(()) };

        self.tree.values.with_mut(node.value, |(path, value), values| {
            let mut children = TreeView::new(path, &mut node.children, values);
            let generator = G::from_value(value, ctx);
            let mut children = ForEach2::new(children, self.traverser, generator);

            // -----------------------------------------------------------------------------
            //   - Traverse children or perform op on current child -
            //   * If the current value is a continer of values (e.g for-loop or control flow)
            //     then it should "bundle" the children with the current level
            // -----------------------------------------------------------------------------
            match self.traverser.traverse(value) {
                true => children.inner_each(ctx, f),
                false => f(ctx, value, children),
            }
        })
    }
}

// -----------------------------------------------------------------------------
//   - Traits -
// -----------------------------------------------------------------------------

trait Traverser<T> {
    fn traverse(&self, input: &mut T) -> bool;
}

/// The generator should generate child nodes, created from a parent node.
pub trait Generator<T, C> {
    fn from_value(value: &mut T, ctx: &mut C) -> Self
    where
        Self: Sized;

    fn generate(&mut self, tree: &mut TreeView<'_, T>, ctx: &mut C) -> bool;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{slab::Key, tree::{root_node, Nodes, Tree, TreeValues}};

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
        Noop,
    }

    // struct Gen<'a, 'bp> {
    //     value: &'a Value<'bp>
    // }

    impl<'frame, 'bp> Generator<Value<'bp>, Ctx<'frame>> for Gen<'bp> {
        fn generate(&mut self, tree: &mut TreeView<'_, Value<'bp>>, ctx: &mut Ctx<'frame>) -> bool {
            match self {
                _ => panic!(),
                // Gen::Loop(counter, body) if *counter > 0 && tree.layout.inner.len() != *counter => {
                //     *counter -= 1;
                //     eprintln!("counter: {counter}");
                //     let value = Value::Iter(body);
                //     tree.insert(tree.offset).commit_child(value);
                // }
                // Gen::Loop(counter, body) => return false,
                // Gen::Single(blueprints) | Gen::Iter(blueprints) => {
                //     if blueprints.is_empty() {
                //         return false;
                //     }

                //     let index = tree.layout.inner.len();
                //     if index == blueprints.len() {
                //         return false;
                //     }

                //     let blueprint = &blueprints[index];
                //     eval(blueprint, tree);
                // }
                // Gen::Noop => return false,
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

    #[test]
    fn runit() {
        let mut tree = Tree::<_>::empty();

        // -----------------------------------------------------------------------------
        //   - This is the final step -
        //   Get blueprints in to the for_each_2
        //   Each nested call needs to carry its own blueprints.
        //   These could sit on the context.
        //
        //   Q: How do we get the blueprints into the context?
        //   A:
        //
        // -----------------------------------------------------------------------------

        // let bp = Blueprint::Single(Single {
        //     value: 0,
        //     children: vec![
        //         Blueprint::Single(Single {
        //             value: 1,
        //             children: vec![Blueprint::Single(Single {
        //                 value: 100,
        //                 children: vec![],
        //             })],
        //         }),
        //         Blueprint::Single(Single {
        //             value: 2,
        //             children: vec![],
        //         }),
        //         Blueprint::Single(Single {
        //             value: 3,
        //             children: vec![],
        //         }),
        //     ],
        // });

        let bp = Blueprint::Single(Single {
            value: 0,
            children: vec![Blueprint::Loop {
                counter: 30,
                body: vec![
                    Blueprint::Single(Single {
                        value: 1,
                        children: vec![],
                    }),
                    Blueprint::Single(Single {
                        value: 2,
                        children: vec![],
                    }),
                ],
            }],
        });

        // -----------------------------------------------------------------------------
        //   - Relevant code here...  -
        // -----------------------------------------------------------------------------

        let mut view = tree.view_mut();
        let mut scopes = vec![];
        let mut ctx = Ctx { scopes: &mut scopes };
        let value_id = eval(&bp, &mut view);

        // TODO: need a sub tree with the children of `value_id`

        let node = view.layout.get_mut(0).unwrap();
        let mut children = TreeView::new(&[0], &mut node.children, &mut view.values);
        view.with_value_mut(value_id, |_path, value, view| {
            let mut for_each = ForEach2::new(view, &Tr);
            let mut size = 0;
            for_each.each(&mut ctx, |ctx, value, children| {
                eprintln!("{value:?}");
                // eprintln!("starting layout for {value:?}");
                size += value.layout(ctx, children);
                // eprintln!("ending layout for {value:?}");
                ControlFlow::Continue(())
            });
        });


        panic!();
        // let (nodes, values) = tree.split();
        // // print_tree(nodes, values, 0);
        // panic!();

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
