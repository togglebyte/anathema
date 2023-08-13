use anathema_values::{ValueRef, Store};

use crate::expression::ControlFlowExpr;
use crate::{Expression, NodeKind};

// fn control_flow() -> Vec<ControlFlow<&'static str, u32>> {
//     vec![
//         ControlFlow {
//             cond: Cond::If(Value::Static(1)),
//             body: vec![
//                 Expression::Node { context: "truthy", children: vec![].into() },
//             ].into(),
//         },
//         ControlFlow {
//             cond: Cond::Else(Some(Value::Static(1))),
//             body: vec![
//                 Expression::Node { context: "else cond", children: vec![].into() },
//             ].into(),
//         },
//         ControlFlow {
//             cond: Cond::Else(None),
//             body: vec![
//                 Expression::Node { context: "else no cond", children: vec![].into() },
//             ].into(),
//         },
//     ]
// }

// pub(crate) fn expressions() -> (Vec<Expression<&'static str, u32>>, Bucket<u32>) {
//     const ITEM: usize = 0;
//     const LIST: usize = 1;
//     const TRUTH: usize = 2;
//     const ELSE_TRUTH: usize = 3;

//     let mut bucket = Bucket::empty();
//     {
//         let mut bucket = bucket.write();
//         bucket.get("item"); // ensure that the paths exists with the correct numbers 
//         bucket.insert_at_path("list", vec![1, 2]); 
//         bucket.insert_at_path("truthy", 1);
//         bucket.insert_at_path("falsey", 0);
//     }

//     let expressions = vec![
//         Expression::Node {
//             context: "root",
//             children: vec![
//                 Expression::Node { context: "first", children: vec![].into() },
//                 Expression::Node { context: "second", children: vec![].into() },
//             ].into()
//         },

//         Expression::Loop {
//             binding: ITEM.into(),
//             collection: Value::Dynamic(LIST.into()),
//             body: vec![
//                 Expression::Loop {
//                     binding: ITEM.into(),
//                     collection: Value::Dynamic(LIST.into()),
//                     body: vec![
//                         Expression::Node { context: "inner loopy child 1", children: vec![].into() },
//                         Expression::Node { context: "inner loopy child 2", children: vec![].into() },
//                     ].into()
//                 },

//                 Expression::Node { context: "loopy child 1", children: vec![].into() },
//                 Expression::Node { context: "loopy child 2", children: vec![].into() },
//             ].into()
//         },

//         Expression::ControlFlow(control_flow().into()),

//         Expression::Node {
//             context: "last",
//             children: vec![
//                 Expression::Node { context: "inner last", children: vec![].into() }
//             ].into()
//         },
//     ];

//     (expressions, bucket)
// }
