use anathema_templates::blueprints::Blueprint;

pub use self::component::ExternalState;
pub use self::element::Element;
use self::eval::{ComponentEval, ControlFlowEval, EvalContext, Evaluator, ForLoopEval, SingleEval};
pub use self::future::try_resolve_future_values;
pub use self::stringify::Stringify;
pub use self::update::update_tree;
use crate::error::Result;
use crate::WidgetTree;

mod component;
mod controlflow;
pub(crate) mod element;
pub(crate) mod eval;
mod future;
pub(crate) mod loops;
mod stringify;
mod update;

#[derive(Debug)]
pub enum WidgetKind<'bp> {
    Element(Element<'bp>),
    For(loops::For<'bp>),
    Iteration(loops::Iteration<'bp>),
    ControlFlow(controlflow::ControlFlow),
    If(controlflow::If<'bp>),
    Else(controlflow::Else<'bp>),
    Component(component::Component<'bp>),
}

pub fn eval_blueprint<'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    parent: &[u16],
    tree: &mut WidgetTree<'bp>,
) -> Result<()> {
    match blueprint {
        Blueprint::Single(single) => SingleEval.eval(single, ctx, parent, tree),
        Blueprint::For(for_loop) => ForLoopEval.eval(for_loop, ctx, parent, tree),
        Blueprint::ControlFlow(flow) => ControlFlowEval.eval(flow, ctx, parent, tree),
        Blueprint::Component(component) => ComponentEval.eval(component, ctx, parent, tree),
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{List, Map, States, Subscriber, Value};
    use anathema_templates::{Expression, Globals};

    use crate::expressions::eval_collection;
    use crate::scope::ScopeLookup;
    use crate::values::ValueId;
    use crate::Scope;

    #[test]
    fn scope_lookup_over_a_collection() {
        // for val in list
        //     test val

        let mut states = States::new();
        let mut scope = Scope::new();
        let globals = Globals::new(Default::default());

        // Setup state to contain a list mapped to the key "list"
        let mut state = Map::<List<_>>::empty();
        let list = Value::<List<_>>::from_iter([123u32, 124]);
        state.insert("list", list);
        let state_id = states.insert(Box::new(state));
        scope.insert_state(state_id);

        // When constructing the for-loop a collection needs to be
        // evaluated from an expression and associated with the loop.
        // In this case the expression would be an `Ident("list")`.
        //
        // The ident is used to lookup the collection in the scope:
        let list_expr = Expression::Ident("list".into());
        let for_key = Subscriber::ZERO;

        // Here we are associating the `val` path with the collection, which
        // is either a slice of expressions or a `PendingValue`.
        let collection = eval_collection(&list_expr, &globals, &scope, &states, for_key);

        // Next up the value would be scoped per iteraton, so `val` is pulled out
        // of the collection by an index, and the resulting value
        // is then associated with the scoped name for the collection (in this case: "list")
        for index in 0..2 {
            scope.push();

            // Here we do a lookup of the collection using the supplied index.
            // The value is scoped to the binding (in this case: "val").
            collection.scope(&mut scope, "val", index);

            let widget_key = Subscriber::ONE;
            // Scope::get will recursively lookup the correct value:
            let output = scope
                .get(ScopeLookup::new("val", widget_key), &mut None, &states)
                .unwrap();

            let int = output.load::<u32>().unwrap();
            assert_eq!(int, 123 + index as u32);
            scope.pop();
        }
    }

    #[test]
    fn nested_scope_lookup_over_a_collection() {
        // for list in lists
        //     for val in list
        //         test val

        let mut states = States::new();
        let mut scope = Scope::new();
        let globals = Globals::new(Default::default());

        // Setup state to contain a list mapped to the key "list"
        let mut state = Map::<List<_>>::empty();
        let mut lists = List::<List<_>>::empty();
        let mut list = List::empty();
        list.push_back(123u32);
        list.push_back(124);
        lists.push_back(list);
        state.insert("lists", lists);

        let s = states.insert(Box::new(state));
        scope.insert_state(s);

        let lists_expr = Expression::Ident("lists".into());
        let list_expr = Expression::Ident("list".into());
        let for_key = Subscriber::ZERO;

        let collection = eval_collection(&lists_expr, &globals, &scope, &states, for_key);

        for index in 0..1 {
            scope.push();

            // Scope the value from the collection
            collection.scope(&mut scope, "list", index);

            let for_key = ValueId::ONE;

            // Next up the value would be scoped per iteraton, so `val` is scoped to `(list, index)`
            for index in 0..2 {
                let collection = eval_collection(&list_expr, &globals, &scope, &states, for_key);
                scope.push();
                collection.scope(&mut scope, "val", index);

                let sub = ValueId::ONE;
                let output = scope.get(ScopeLookup::new("val", sub), &mut None, &states).unwrap();

                let int = output.load::<u32>().unwrap();
                assert_eq!(int, 123 + index as u32);
                scope.pop();
            }

            scope.pop();
        }
    }
}
