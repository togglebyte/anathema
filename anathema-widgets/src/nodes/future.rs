use anathema_store::tree::PathFinder;

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
// use super::update::scope_value;
use super::WidgetContainer;
use crate::error::{Error, Result};
use crate::expressions::{eval, eval_collection};
use crate::values::{Collection, ValueId};
use crate::{WidgetKind, WidgetTree};

struct ResolveFutureValues<'a, 'b, 'bp> {
    value_id: ValueId,
    ctx: EvalContext<'a, 'b, 'bp>,
}

impl<'a, 'b, 'bp> PathFinder for ResolveFutureValues<'a, 'b, 'bp> {
    type Input = WidgetContainer<'bp>;
    type Output = Result<()>;

    fn apply(&mut self, node: &mut WidgetContainer<'bp>, path: &[u16], tree: &mut WidgetTree<'bp>) -> Self::Output {
        // if the widget is a component, defer scoping the value until afterwards
        if !matches!(node.kind, WidgetKind::Component(_)) {
            panic!("let's not use this anymore")
            // scope_value(&node.kind, self.ctx.scope, &[]);
        }

        try_resolve_value(&mut node.kind, &mut self.ctx, self.value_id, path, tree)?;

        if matches!(node.kind, WidgetKind::Component(_)) {
            panic!("let's not use this anymore")
            // scope_value(&node.kind, self.ctx.scope, &[]);
        }

        Ok(())
    }

    fn parent(&mut self, parent: &mut WidgetContainer<'bp>, children: &[u16]) {
        panic!("let's not use this anymore")
        // scope_value(&parent.kind, self.ctx.scope, children);
    }
}

pub fn try_resolve_future_values<'bp>(
    value_id: ValueId,
    path: &[u16],
    tree: &mut WidgetTree<'bp>,
    ctx: EvalContext<'_, '_, 'bp>,
) {
    let res = ResolveFutureValues { value_id, ctx };
    tree.apply_path_finder(path, res);
}

fn try_resolve_value<'bp>(
    widget: &mut WidgetKind<'bp>,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    value_id: ValueId,
    path: &[u16],
    tree: &mut WidgetTree<'bp>,
) -> Result<()> {
    match widget {
        WidgetKind::Element(Element { container, .. }) => {
            ctx.attribute_storage
                .with_mut(container.id, |attributes, attribute_storage| {
                    let Some(val) = attributes.get_mut_with_index(value_id.index()) else { return };

                    if let Some(expr) = val.expr {
                        let value = eval(expr, ctx.globals, ctx.scope, ctx.states, attribute_storage, value_id);

                        if val.replace(value) {
                            // TODO: do we need this?
                            ctx.dirty_widgets.push(value_id.key());
                        }
                    }
                });
        }
        WidgetKind::For(for_loop) => {
            panic!()
            // // 1. Assign a new collection
            // // 2. Remove the current children
            // // 3. Build up new children

            // for_loop.collection = eval_collection(
            //     for_loop.collection.expr.unwrap(),
            //     ctx.globals,
            //     ctx.scope,
            //     ctx.states,
            //     ctx.attribute_storage,
            //     value_id,
            // );

            // tree.remove_children(path);

            // let collection = &for_loop.collection;
            // let binding = &for_loop.binding;
            // let body = for_loop.body;
            // let parent = path;

            // for index in 0..collection.count() {
            //     ctx.scope.push();

            //     match collection.inner() {
            //         Collection::Static(expressions) => {
            //             let downgrade = expressions[index].downgrade();
            //             ctx.scope.scope_downgrade(binding, downgrade)
            //         }
            //         Collection::Dyn(value_ref) => {
            //             let value = value_ref
            //                 .as_state()
            //                 .and_then(|state| state.state_lookup(index.into()))
            //                 .expect("the collection has a value since it has a length");
            //             ctx.scope.scope_pending(binding, value)
            //         }
            //         Collection::Index(collection, _) => match &**collection {
            //             Collection::Static(expressions) => {
            //                 let downgrade = expressions[index].downgrade();
            //                 ctx.scope.scope_downgrade(binding, downgrade)
            //             }
            //             Collection::Dyn(value_ref) => {
            //                 let value = value_ref
            //                     .as_state()
            //                     .and_then(|state| state.state_lookup(index.into()))
            //                     .expect("the collection has a value since it has a length");
            //                 ctx.scope.scope_pending(binding, value)
            //             }
            //             Collection::Future => {}
            //             Collection::Index(_, _) => unreachable!("maaybe it's not?"),
            //         },
            //         Collection::Future => {}
            //     }

            //     let iter_id = tree
            //         .insert(parent)
            //         .commit_child(WidgetKind::Iteration(super::loops::Iteration {
            //             loop_index: anathema_state::Value::new(index as i64),
            //             binding,
            //         }))
            //         .ok_or(Error::TreeTransactionFailed)?;

            //     // Scope the iteration value
            //     tree.with_value_mut(iter_id, |parent, widget, tree| {
            //         let WidgetKind::Iteration(iter) = widget else { unreachable!() };
            //         ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

            //         for bp in body {
            //             crate::eval_blueprint(bp, ctx, parent, tree)?;
            //         }
            //         Ok(())
            //     })?;

            //     ctx.scope.pop();
            // }
        }
        // WidgetKind::If(widget) => {
        //     if let Some(expr) = widget.cond.expr {
        //         let value = eval(
        //             expr,
        //             ctx.globals,
        //             ctx.scope,
        //             ctx.states,
        //             ctx.attribute_storage,
        //             value_id,
        //         );
        //         widget.cond = value;
        //     }
        // }
        // WidgetKind::Else(el) => {
        //     let Some(val) = &mut el.cond else { return Ok(()) };
        //     if let Some(expr) = val.expr {
        //         *val = eval(
        //             expr,
        //             ctx.globals,
        //             ctx.scope,
        //             ctx.states,
        //             ctx.attribute_storage,
        //             value_id,
        //         );
        //     }
        // }
        WidgetKind::ControlFlow(_) => unreachable!(),
        WidgetKind::ControlFlowContainer(_) => unreachable!(),
        WidgetKind::Iteration(_) => unreachable!(),
        WidgetKind::Component(component) => {
            ctx.attribute_storage
                .with_mut(component.widget_id, |attributes, attribute_storage| {
                    let Some(val) = attributes.get_mut_with_index(value_id.index()) else { return };

                    if let Some(expr) = val.expr {
                        let value = eval(expr, ctx.globals, ctx.scope, ctx.states, attribute_storage, value_id);
                        val.replace(value);
                    }
                });
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use anathema_state::{drain_futures, States};
    use anathema_store::stack::Stack;
    use anathema_templates::expressions::{ident, index, list, map, num, strlit};
    use anathema_templates::{Expression, Globals};

    use super::*;
    use crate::{AttributeStorage, Scope};

    fn future_value(expr: &Expression, value_id: ValueId) {
        let globals = Globals::default();
        let scope = Scope::new();
        let states = States::new();
        let attributes = AttributeStorage::empty();
        let mut futures = Stack::empty();

        drain_futures(&mut futures);
        assert_eq!(futures.len(), 0);

        eval(expr, &globals, &scope, &states, &attributes, value_id);

        drain_futures(&mut futures);
        assert_eq!(futures.len(), 1);
        let x = futures.pop().unwrap();
        assert_eq!(x, value_id);
    }

    #[test]
    fn ident_future() {
        let expr = ident("val");
        let value_id = ValueId::ONE;
        future_value(&expr, value_id);
    }

    #[test]
    fn list_future() {
        let expr = index(list([num(1), num(2)]), ident("notfound"));
        let value_id = ValueId::ONE;
        future_value(&expr, value_id);
    }

    #[test]
    fn map_future() {
        let expr = index(map([("key", strlit("val"))]), ident("key"));
        let value_id = ValueId::ONE;
        future_value(&expr, value_id);
    }

    #[test]
    fn resolve_ident_future() {
        // // Template:
        // // text some_map.collection[some_value]

        // let expr = index(index(ident("some_map"), strlit("collection")), ident("some_value"));

        // let value_id = ValueId::ONE;

        // let globals = Globals::default();
        // let mut scope = Scope::new();
        // let mut states = States::new();
        // let mut futures = Stack::empty();
        // // let ret = eval(&expr, &globals, &scope, &states, value_id);

        // // drain_futures(&mut futures);
        // // assert_eq!(futures.len(), 1);
        // // futures.clear();

        // // let ret = eval(&expr, &globals, &scope, &states, value_id);
        // // assert_eq!(*ret, EvalValue::Empty);
        // // futures.clear();
        // // assert_eq!(futures.len(), 0);

        // let mut some_map = Map::<List<_>>::empty();
        // let mut list = List::empty();
        // list.push_back(123u32);
        // some_map.insert("collection", list);

        // let mut state = Map::<Map<_>>::empty();
        // state.insert("some_map", some_map);
        // let sid1 = states.insert(Box::new(state));
        // scope.insert_state(sid1);

        // let mut state = Map::<usize>::empty();
        // state.insert("some_value", 0);

        // let sid2 = states.insert(Box::new(state));
        // scope.insert_state(sid2);

        // let lookup = ScopeLookup::new("some_map", Some(value_id));

        // let foolery = scope.get(lookup, &mut None, &states);

        // let ret = eval(&expr, &globals, &scope, &states, value_id);
        // assert_eq!(*ret, EvalValue::Empty);

        // drain_futures(&mut futures);
        // assert_eq!(futures.len(), 1);
    }
}
