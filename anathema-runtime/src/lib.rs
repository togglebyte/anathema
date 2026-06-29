// -----------------------------------------------------------------------------
//   - Macro requirements -
// -----------------------------------------------------------------------------
#[allow(unused_extern_crates)]
extern crate self as anathema;
use std::fmt::Display;
use std::io::Stdout;

use anathema_compiler::Variables;
pub use anathema_compiler::blueprints::Blueprint;
pub use anathema_compiler::expressions::ExpressionId;
use anathema_compiler::expressions::Expressions;
pub use anathema_compiler::{Color, FromColor};
use anathema_geometry::Pos;
pub use anathema_state_derive::State;
use anathema_store::slab::Key;

pub use self::states::State;
// -----------------------------------------------------------------------------
//   - Exports for proc macro -
// -----------------------------------------------------------------------------
#[allow(unused_imports)]
pub use crate as state;
// -----------------------------------------------------------------------------
//   -  -
// -----------------------------------------------------------------------------
use crate::attributes::AttributeRegistry;
pub use crate::attributes::{Attributes, WidgetAttributes};
pub use crate::components::{Component, ComponentId, Components};
pub use crate::constraints::Constraints;
pub use crate::elements::ElementId;
use crate::elements::{Element, Elements};
use crate::eval::blueprints::BlueprintEvalCtx;
use crate::eval::blueprints::expression::RuntimeExpressions;
use crate::eval::blueprints::scope::Scope;
pub use crate::eval::blueprints::values::TemplateValue;
pub use crate::functions::FunctionTable;
pub use crate::runtime::Runtime;
pub use crate::states::{AnyList, AnyMap, TypeId};
use crate::testing::test_widgets;
pub use crate::value::{AnonValue, Type};
use crate::widgets::iter::WidgetRef;
use crate::widgets::{Children, Layouts, RegisteredWidgets};

mod attributes;
mod components;
mod constraints;
mod elements;
mod error;
mod eval;
pub(crate) mod functions;
mod paint;
mod runtime;
pub mod states;
mod testing;
pub mod value;
pub mod widgets;

// pub struct Temp<'bp> {
//     blueprint: &'bp Blueprint,
//     expressions: &'bp Expressions,
//     registered_widgets: RegisteredWidgets,
//     elements: Elements<'bp>,
//     attributes: AttributeRegistry<'bp>,
//     components: Components,
//     variables: Variables,
//     runtime_expressions: RuntimeExpressions<'bp>,
//     functions: &'bp FunctionTable,
//     scope: Scope<'bp>,
//     dirty_elements: Vec<ElementId>,
//     layouts: Layouts,
//     fe: Crossterm<Stdout>,
// }

// impl<'bp> Temp<'bp> {
//     pub fn new(
//         variables: Variables,
//         blueprint: &'bp Blueprint,
//         expressions: &'bp Expressions,
//         functions: &'bp FunctionTable,
//         registered_widgets: RegisteredWidgets,
//     ) -> Self {
//         Self {
//             blueprint,
//             expressions,
//             registered_widgets,
//             // registered_widgets: test_widgets(),
//             elements: Elements::empty(),
//             attributes: AttributeRegistry::empty(),
//             components: Components::empty(),
//             variables,
//             runtime_expressions: RuntimeExpressions::empty(),
//             functions,
//             scope: Scope::empty(),
//             dirty_elements: vec![],
//             layouts: Layouts::empty(),
//             fe: Crossterm::new(),
//         }
//     }

//     pub fn ctx<'frame>(&'frame mut self) -> (EvalCtx<'frame, 'bp>, &'bp Blueprint, &'frame RegisteredWidgets) {
//         let ctx = EvalCtx::new(
//             &mut self.elements,
//             &mut self.attributes,
//             &mut self.components,
//             &self.variables,
//             self.expressions,
//             self.functions,
//             &mut self.scope,
//             &mut self.runtime_expressions,
//             &mut self.dirty_elements,
//         );

//         (ctx, self.blueprint, &self.registered_widgets)
//     }

//     pub fn eval(&mut self) {
//         let (mut ctx, bp, registered_widgets) = self.ctx();
//         eval::eval(bp, &mut ctx, registered_widgets, None).unwrap();
//     }

//     pub fn display(&mut self) {
//         let mut id = self.elements.root;

//         loop {
//             let node = &self.elements[id];
//             match &node.element {
//                 Element::For { binding, collection } => todo!(),
//                 Element::Iteration { loop_counter } => todo!(),
//                 Element::ControlFlow => todo!(),
//                 Element::Condition(remote_cell) => todo!(),
//                 Element::With => todo!(),
//                 Element::Widget(widget) => {
//                     let attributes = &self.attributes[id];
//                     let constraints = Constraints::new(self.fe.size());

//                     // Layout
//                     let children = Children::new(&node.children, &self.elements, &self.attributes);
//                     let widget_attributes =
//                         WidgetAttributes::new(&self.elements, node.parent, attributes, &self.attributes);
//                     let widget_ref = WidgetRef::new(id, widget.borrow_mut(), widget_attributes, children);
//                     widget_ref.layout(&mut self.layouts, constraints);

//                     // Position
//                     let children = Children::new(&node.children, &self.elements, &self.attributes);
//                     let widget_attributes =
//                         WidgetAttributes::new(&self.elements, node.parent, attributes, &self.attributes);
//                     let widget_ref = WidgetRef::new(id, widget.borrow_mut(), widget_attributes, children);
//                     widget_ref.position(&mut self.layouts, Pos::ZERO);

//                     // Paint
//                     let children = Children::new(&node.children, &self.elements, &self.attributes);
//                     let widget_attributes =
//                         WidgetAttributes::new(&self.elements, node.parent, attributes, &self.attributes);
//                     let widget_ref = WidgetRef::new(id, widget.borrow_mut(), widget_attributes, children);
//                     widget_ref.paint(&mut self.fe, &self.layouts);

//                     break;
//                 }
//                 Element::Component(component_id) => todo!(),
//             }
//         }

//         self.fe.render();
//     }
// }
