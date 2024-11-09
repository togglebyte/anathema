// use super::{Node, Tree, TreeValues};
// use crate::slab::GenSlab;

// enum Value {
//     S(String),
//     I(usize),
// }

// pub struct Each<T>
// where
//     T: Children,
// {
//     inner: T,
//     transformer: T::Transformer,
// }

// impl<T> Each<T>
// where
//     T: Children,
// {
//     fn for_each<F>(&mut self, f: F, tree: &mut Tree<Value>)
//     where
//         for<'a> F: FnMut(&mut usize, Self),
//     {
//         // for node in tree.layout.iter() {
//         //     // filter and use tree
//         //     tree.with_value_mut(node.value, |path, value, tree| {
//         //         let thing = self.transformer.transform(value);
//         //         // self.inner.next(thing);
//         //     });

//         // }
//     }
// }

// pub trait Children {
//     type Transformer: Transformer;

//     fn next(&mut self);
// }

// pub trait Transformer {
//     type Context;

//     fn new() -> Self
//     where
//         Self: Sized;

//     fn transform(&mut self, input: &mut Value) -> &mut usize;
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::tree::{root_node, Tree};

//     struct Evaluator;

//     impl Transformer for Evaluator {
//         type Context = ();
//         type From<'a> = &'a String;
//         type To<'a> = Result<usize, ()>;

//         fn new() -> Self
//         where
//             Self: Sized,
//         {
//             Self
//         }

//         fn transform<'a>(&mut self, input: Self::From<'a>) -> Self::To<'a> {
//             input.parse().map_err(|_| ())
//         }
//     }

//     struct Zero;

//     impl Children for Zero {
//         type Transformer = Evaluator;

//         fn next(&mut self) {
//             todo!()
//         }
//     }

//     #[test]
//     fn doit() {
//         let mut tree = Tree::<String>::empty();
//         tree.insert(root_node()).commit_child("2".into());
//         let path = &[0];
//         tree.insert(path).commit_child("123".into());
//         let path = &[0, 0];
//         tree.insert(path).commit_child("1".into());
//         tree.insert(path).commit_child("broken number".into());

//         let mut each = Each {
//             inner: Zero,
//             transformer: Evaluator,
//         };
//         each.for_each(|node, children| {
//             panic!();
//         });

//         // panic!("{tree:#?}");

//         // assert_eq!(expected, actual);
//     }
// }
