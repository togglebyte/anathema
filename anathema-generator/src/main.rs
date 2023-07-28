fn main() {

}
// use anathema_generator::*;
// use anathema_values::*;

// enum Value {
//     N(usize),
// }

// enum Template {
//     Node {
//         ident: String,
//         children: Vec<Template>,
//     },
//     ForLoop {
//         binding: PathId,
//         collection: PathId,
//         body: Vec<Template>,
//     },
// }

// struct Context {
//     ident: String,
// }

// fn templates_to_expression_tree() -> Vec<Expression<Context, usize>> {
//     let templates = vec![
//         Template::Node {
//             ident: "root".into(),
//             children: vec![
//                 Template::Node { ident: "first".into(), children: vec![] }
//             ]
//         },
//         Template::ForLoop { 
//             binding: 0.into(),
//             collection: 0.into(), body: vec![
//                 Template::Node { ident: "loopy child 1".into(), children: vec![] },
//                 Template::Node { ident: "loopy child 2".into(), children: vec![] },
//             ]
//         },
//         Template::Node { ident: "last".into(), children: vec![] }
//     ];

//     templates.into_iter().map(template_to_expression).collect()
// }

// fn template_to_expression(template: Template) -> Expression<Context, usize> {
//     match template {
//         Template::Node { ident, children } => {
//             let children = children.into_iter().map(template_to_expression).collect();
//             Expression::Node {
//                 context: Context { ident },
//                 children,
//             }
//         }
//         Template::ForLoop { binding, collection, body } => Expression::Loop {
//             collection,
//             binding,
//             body: body.into_iter().map(template_to_expression).collect()
//         }
//     }
// }

// fn main() {
//     let expressions = templates_to_expression_tree();
//     let mut nodes = Nodes::new(expressions);
// }

// // fn template_to_widget(template: &Template) -> Option<Widget> {
// //     match template {
// //         Template::Node { ident, children } => Some(Widget {
// //             ident: ident.clone(),
// //             children: Nodes::new(children),
// //         }),
// //         _ => None,
// //     }
// // }

// // struct Widget {
// //     ident: String,
// //     children: Nodes<Template, Self>,
// // }

// // impl Widget {
// //     fn layout(&mut self, bucket: &BucketRef<'_, Value>) {
// //         let children = self.children.gen(bucket, template_to_widget);

// //         // while let Some(widget) = children.next() {
// //         // }
// //     }

// //     fn update(&mut self) {}
// // }

// // fn main() {
// //     // Values
// //     let mut bucket = Bucket::<usize>::empty();
// //     bucket.write().insert("a", 1);
// //     bucket.write().insert("b", vec![2, 3, 4]);
// //     let collection: PathId = bucket.read().get_path_unchecked("b");

// //     // Templates
// //     let templates = vec![1, 2, 3, usize::MAX];

// //     // Expressions
// //     let expressions = vec![
// //         Expression::Single(&templates[0]),
// //         Expression::Loop {
// //             collection,
// //             body: &templates[1..2],
// //         },
// //         Expression::Single(&templates[2]),
// //     ];

// //     // Generator
// //     // let mut generator = Generator::new(bucket.read(), expressions, |t| Some(*t));

// //     let mut nodes = Nodes::<_, Widget>::new(templates);

// //     // let mut gen = nodes.gen();
// //     // while let Some(node) = gen.next() {
// //     // }
// // }
