use anathema_store::smallmap::SmallIndex;
use anathema_store::tree::Generator;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::Scope;
use loops::LOOP_INDEX;

pub use self::element::Element;
use self::eval::{ComponentEval, ControlFlowEval, Evaluator, ForLoopEval, SingleEval};
// pub use self::future::try_resolve_future_values;
pub use self::stringify::Stringify;
pub use self::update::update_widget;
use crate::error::Result;
use crate::expressions::ExprEvalCtx;
use crate::layout::{EvalCtx, LayoutCtx};
use crate::values::ValueId;
use crate::widget::WidgetTreeView;
use crate::{WidgetId, WidgetTree};

pub(crate) mod component;
pub(crate) mod controlflow;
pub(crate) mod element;
pub(crate) mod eval;
mod future;
pub(crate) mod loops;
mod stringify;
mod update;

// -----------------------------------------------------------------------------
//   - Generators -
// -----------------------------------------------------------------------------
pub enum WidgetGenerator<'bp> {
    Children(&'bp [Blueprint]),
    Single,
    Loop(&'bp [Blueprint]),
    ControlFlow,
    Noop,
}

impl<'rt, 'bp> Generator<WidgetContainer<'bp>, EvalCtx<'rt, 'bp>> for WidgetGenerator<'bp> {
    fn from_value(value: &mut WidgetContainer<'bp>, ctx: &mut EvalCtx<'rt, 'bp>) -> Self
    where
        Self: Sized,
    {
        match &value.kind {
            WidgetKind::Element(_) | WidgetKind::ControlFlowContainer(_) => WidgetGenerator::Children(value.children),
            WidgetKind::For(for_loop) => WidgetGenerator::Loop(value.children),
            WidgetKind::Iteration(iter) => todo!(),
            WidgetKind::ControlFlow(cf) => todo!(),
            WidgetKind::Component(_) => todo!(),
        }
    }

    fn generate(&mut self, tree: &mut WidgetTreeView<'_, 'bp>, ctx: &mut EvalCtx<'_, 'bp>) -> bool {
        match self {
            WidgetGenerator::Children(blueprints) => {
                if blueprints.is_empty() {
                    return false;
                }

                let index = tree.layout_len();
                if index >= blueprints.len() {
                    return false;
                }

                let blueprint = &blueprints[index];

                let parent = tree.offset;
                eval_blueprint(blueprint, ctx, panic!("missing scope"), parent, tree);
                true
            }
            WidgetGenerator::Single => todo!(),
            WidgetGenerator::Loop(_) => todo!(),
            WidgetGenerator::ControlFlow => todo!(),
            WidgetGenerator::Noop => false,
        }
    }
}

#[derive(Debug)]
pub enum WidgetKind<'bp> {
    Element(Element<'bp>),
    For(loops::For<'bp>),
    Iteration(loops::Iteration<'bp>),
    ControlFlow(controlflow::ControlFlow<'bp>),
    ControlFlowContainer(u16),
    // If(controlflow::If<'bp>),
    // Else(controlflow::Else<'bp>),
    Component(component::Component<'bp>),
}

#[derive(Debug)]
pub struct WidgetContainer<'bp> {
    pub kind: WidgetKind<'bp>,
    pub(crate) children: &'bp [Blueprint],
}

impl<'bp> WidgetContainer<'bp> {
    pub fn new(kind: WidgetKind<'bp>, blueprints: &'bp [Blueprint]) -> Self {
        Self {
            kind,
            children: blueprints,
        }
    }

    pub(crate) fn resolve_pending_values(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, widget_id: WidgetId) {
        ctx.changes(
            widget_id,
            |attributes, expr_eval_ctx, strings, value_id| match &mut self.kind {
                WidgetKind::Element(element) => {
                    let Some(value) = attributes.get_mut_with_index(value_id.index()) else { return };
                    value.reload_val(value_id, expr_eval_ctx, strings);
                }
                WidgetKind::For(for_loop) => {
                    for_loop.collection.reload_val(value_id, expr_eval_ctx, strings);
                }
                WidgetKind::ControlFlow(controlflow) => {
                    for value in controlflow.elses.iter_mut().filter_map(|e| e.cond.as_mut()) {
                        value.reload_val(value_id, expr_eval_ctx, strings);
                    }
                }
                WidgetKind::ControlFlowContainer(_) => (),
                WidgetKind::Iteration(iteration) => (),
                WidgetKind::Component(component) => (),
            },
        );
    }
}

pub fn eval_blueprint<'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalCtx<'_, 'bp>,
    scope: &Scope<'_, 'bp>,
    parent: &[u16],
    tree: &mut WidgetTreeView<'_, 'bp>,
) -> Result<()> {
    match blueprint {
        Blueprint::Single(single) => SingleEval.eval(single, ctx, scope, parent, tree),
        Blueprint::For(for_loop) => ForLoopEval.eval(for_loop, ctx, scope, parent, tree),
        Blueprint::ControlFlow(flow) => ControlFlowEval.eval(flow, ctx, scope, parent, tree),
        Blueprint::Component(component) => ComponentEval.eval(component, ctx, scope, parent, tree),
    }
}

// #[cfg(test)]
// mod test {
//     use anathema_state::{List, Map, States, Subscriber};
//     use anathema_templates::expressions::{ident, index, strlit};
//     use anathema_templates::Globals;

//     use crate::expressions::{eval_collection, Resolver};
//     use crate::scope::ScopeLookup;
//     use crate::values::ValueId;
//     use crate::{AttributeStorage, Scope};

//     #[test]
//     fn scope_lookup_over_a_collection() {
//         // for val in state.list
//         //     test val

//         let mut states = States::new();
//         let mut scope = Scope::new();
//         let attributes = AttributeStorage::empty();
//         let globals = Globals::new(Default::default());

//         // Setup state to contain a list mapped to the key "list"
//         let mut state = Map::<List<_>>::empty();
//         let list = List::from_iter([123u32, 124]);
//         state.insert("list", list);
//         let state_id = states.insert(Box::new(state));
//         scope.insert_state(state_id);

//         // When constructing the for-loop a collection needs to be
//         // evaluated from an expression and associated with the loop.
//         // In this case the expression would be an `Ident("list")`.
//         //
//         // The ident is used to lookup the collection in the scope:
//         let list_expr = index(ident("state"), strlit("list"));
//         let for_key = Subscriber::ZERO;

//         // Here we are associating the `val` path with the collection, which
//         // is either a slice of expressions or a `PendingValue`.
//         let collection = eval_collection(&list_expr, &globals, &scope, &states, &attributes, for_key);

//         // Next up the value would be scoped per iteraton, so `val` is pulled out
//         // of the collection by an index, and the resulting value
//         // is then associated with the scoped name for the collection (in this case: "list")
//         for index in 0..2 {
//             scope.push();

//             // Here we do a lookup of the collection using the supplied index.
//             // The value is scoped to the binding (in this case: "val").
//             collection.scope(&mut scope, "val", index);

//             let widget_key = Subscriber::ONE;
//             // Scope::get will recursively lookup the correct value:
//             let output = scope
//                 .get(ScopeLookup::new("val", widget_key), &mut None, &states)
//                 .unwrap();

//             let int = output.load::<u32>().unwrap();
//             assert_eq!(int, 123 + index as u32);
//             scope.pop();
//         }
//     }

//     #[test]
//     fn nested_scope_lookup_over_a_collection() {
//         // for list in state.lists
//         //     for val in list
//         //         test val

//         let mut states = States::new();
//         let attributes = AttributeStorage::empty();
//         let mut scope = Scope::new();
//         let globals = Globals::new(Default::default());

//         // Setup state to contain a list mapped to the key "list"
//         let mut state = Map::<List<_>>::empty();
//         let mut lists = List::<List<_>>::empty();
//         let list = List::from_iter(123..=124u32);
//         lists.push_back(list);
//         state.insert("lists", lists);

//         let s = states.insert(Box::new(state));
//         scope.insert_state(s);

//         let lists_expr = index(ident("state"), strlit("lists"));
//         let list_expr = ident("list");
//         let for_key = Subscriber::ZERO;

//         let collection = eval_collection(&lists_expr, &globals, &scope, &states, &attributes, for_key);

//         for idx in 0..1 {
//             scope.push();

//             // Scope the value from the collection
//             collection.scope(&mut scope, "list", idx);

//             let for_key = ValueId::ONE;

//             // Next up the value would be scoped per iteraton, so `val` is scoped to `(list, index)`
//             for idx in 0..2 {
//                 let collection = eval_collection(&list_expr, &globals, &scope, &states, &attributes, for_key);
//                 scope.push();
//                 collection.scope(&mut scope, "val", idx);

//                 let expr = ident("val");
//                 let output = Resolver::root(&scope, &states, &attributes, &globals, for_key, false).resolve(&expr);
//                 let int = output.load::<u32>().unwrap();
//                 assert_eq!(int, 123 + idx as u32);
//                 scope.pop();
//             }

//             scope.pop();
//         }
//     }
// }
