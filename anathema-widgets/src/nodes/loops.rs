use anathema_state::Change;
use anathema_store::tree::new_node_path;
use anathema_templates::blueprints::Blueprint;

use super::{WidgetContainer, WidgetKind};
use crate::error::{Error, Result};
use crate::expressions::eval_collection;
use crate::scope::Scope;
use crate::values::{Collection, ValueId};
use crate::widget::WidgetTreeView;
use crate::{eval_blueprint, AttributeStorage, Value, WidgetId, WidgetTree};

pub(super) const LOOP_INDEX: &str = "loop";

#[derive(Debug)]
pub struct For<'bp> {
    pub(crate) binding: &'bp str,
    pub(crate) collection: Value<'bp, Collection<'bp>>,
    // TODO: remove the body here as it's attached to the container
    pub(crate) body: &'bp [Blueprint],
}

impl<'bp> For<'bp> {
    pub(super) fn scope_value(&self, scope: &mut Scope<'bp>, index: usize) {
        panic!("don't use this, this is for the old collections. The Iter should do this part")
        // self.collection.scope(scope, self.binding, index)
    }

    pub(crate) fn collection(&self) -> &Collection<'_> {
        self.collection.inner()
    }

    pub(super) fn update(
        &mut self,
        // ctx: &mut EvalContext<'_, '_, 'bp>,
        change: &Change,
        value_id: ValueId,
        mut tree: WidgetTreeView<'_, 'bp>,
    ) -> Result<()> {
        match change {
            Change::Inserted(index, value) => {
                // 1. Declare insert path
                // 2. Create new iteration
                // 3. Insert new iteration
                // 4. Update index of all subsequent iterations

                // ctx.scope.push();
                // ctx.scope.scope_pending(self.binding, *value);

                let path = [*index as u16];
                let transaction = tree.insert(&path);
                let widget = WidgetKind::Iteration(Iteration {
                    loop_index: anathema_state::Value::new(*index as i64),
                    binding: self.binding,
                });
                let widget = WidgetContainer::new(widget, &self.body);
                let _ = transaction.commit_at(widget).ok_or(Error::TreeTransactionFailed)?;

                for child in &tree.layout[*index as usize + 1..] {
                    let iter_widget = tree.values.get_mut(child.value());
                    let Some((
                        _,
                        WidgetContainer {
                            kind: WidgetKind::Iteration(iter),
                            ..
                        },
                    )) = iter_widget
                    else {
                        unreachable!()
                    };
                    *iter.loop_index.to_mut() += 1;
                }
            }
            Change::Removed(index) => {
                let path = [*index as u16];
                let child_to_remove = new_node_path(&path, *index as u16);
                tree.relative_remove(&[*index as u16]);
            }
            Change::Dropped => {
                tree.relative_remove(&[]);

                // // TODO unwrap, ewww
                // self.collection = eval_collection(
                //     self.collection.expr.unwrap(), // Map None to an error
                //     ctx.globals,
                //     ctx.scope,
                //     ctx.states,
                //     ctx.attribute_storage,
                //     value_id,
                // );

                // for index in 0..self.collection.count() {
                //     self.scope_value(ctx.scope, index);
                //     ctx.scope.push();

                //     let widget = WidgetKind::Iteration(Iteration {
                //             loop_index: anathema_state::Value::new(index as i64),
                //             binding: self.binding,
                //         });

                //     let iter_id = tree
                //         .insert(path)
                //         .commit_child()
                //         .ok_or(Error::TreeTransactionFailed)?;

                //     // Scope the iteration value
                //     tree.with_value_mut(iter_id, |parent, widget, tree| -> Result<()> {
                //         let WidgetKind::Iteration(iter) = widget else { unreachable!() };
                //         ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

                //         for bp in self.body {
                //             eval_blueprint(bp, ctx, parent, tree)?;
                //         }

                //         Ok(())
                //     })?;

                //     ctx.scope.pop();
                // }
            }
            Change::Changed => {
                // TODO implement this as an optimisation once the runtime is done.
                //      Use this to flag the element as needs-layout.
                //      Every element that needs layout should apply
                //      layout using cached constraints and size.
                //      If the size changes, then this has to be propagate
                //      throughout the widget tree
                //
                // NOTE: This comment was written 12,000 years ago, is this still relevant?
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Iteration<'bp> {
    pub loop_index: anathema_state::Value<i64>,
    pub binding: &'bp str,
}

// #[cfg(test)]
// mod test {
//     use anathema_state::{drain_changes, Changes, List, Map, StateId, States};
//     use anathema_store::tree::root_node;
//     use anathema_templates::Document;

//     use super::*;
//     use crate::components::ComponentRegistry;
//     use crate::nodes::stringify::Stringify;
//     use crate::nodes::eval_blueprint;
//     use crate::testing::setup_test_factory;
//     use crate::{AttributeStorage, Components, DirtyWidgets, FloatingWidgets};

//     #[test]
//     fn loop_remove() {
//         let mut list = List::empty();
//         list.push_back(1u32);
//         list.push_back(2u32);
//         list.push_back(3u32);

//         let mut map = Map::<List<_>>::empty();
//         map.insert("a", list);

//         let tpl = "
//         for x in state.a
//             test x
//             // test loop
//         ";

//         let (blueprint, globals) = Document::new(tpl).compile().unwrap();
//         let mut widget_tree = WidgetTree::empty();
//         let mut attribute_storage = AttributeStorage::empty();
//         let mut floating_widgets = FloatingWidgets::empty();
//         let factory = setup_test_factory();
//         let mut component_registry = ComponentRegistry::new();
//         let mut components = Components::new();
//         let mut dirty_widgets = DirtyWidgets::empty();
//         let mut states = States::new();
//         let state_id = states.insert(Box::new(map));
//         let mut scope = Scope::new();
//         scope.insert_state(state_id);
//         let mut ctx = EvalContext::new(
//             &globals,
//             &factory,
//             &mut scope,
//             &mut states,
//             &mut component_registry,
//             &mut attribute_storage,
//             &mut floating_widgets,
//             &mut components,
//             &mut dirty_widgets,
//         );

//         eval_blueprint(&blueprint, &mut ctx, &[], &mut widget_tree).unwrap();

//         let mut stringify = Stringify::new(&attribute_storage);
//         widget_tree.apply_visitor(&mut stringify);
//         let output = stringify.finish();

//         println!("{output}");

//         {
//             let map = states.get_mut(StateId::ZERO).unwrap();
//             let map = map
//                 .to_any_mut()
//                 .downcast_mut::<anathema_state::Value<Map<List<u32>>>>()
//                 .unwrap();
//             let mut map = map.to_mut();
//             let list = map.get_mut("a").unwrap();
//             list.insert(0, 9); // 9, 1, 2, 3,
//             list.insert(0, 10); // 10, 9, 1, 2, 3
//             list.push_back(100); // 10, 9, 1, 2, 3, 100
//             list.push_back(101); // 10, 9, 1, 2, 3, 100, 101
//             list.push_back(102); // 10, 9, 1, 2, 3, 100, 101, 102
//             list.insert(0, 8); // 8, 10, 9,  1, 2, 3, 100, 101, 102
//             list.remove(0); // 10, 9, 1, 2, 3, 100, 101, 102
//             list.remove(0); // 9, 1, 2, 3, 100, 101, 102
//         }

//         eprintln!("---------------");
//         let mut local_changes = Changes::empty();
//         drain_changes(&mut local_changes);
//         local_changes.drain().rev().for_each(|(subs, change)| {
//             subs.with(|sub| {
//                 eprintln!("- apply change: {change:?}");
//                 let mut scope = Scope::with_capacity(10);
//                 let widget_path = widget_tree.path(sub);

//                 let ctx = EvalContext::new(
//                     &globals,
//                     &factory,
//                     &mut scope,
//                     &mut states,
//                     &mut component_registry,
//                     &mut attribute_storage,
//                     &mut floating_widgets,
//                     &mut components,
//                     &mut dirty_widgets,
//                 );

//                 update_tree(&change, sub, &widget_path, &mut widget_tree, ctx);
//             });
//         });

//         let mut scope = Scope::new();
//         scope.insert_state(state_id);

//         let mut stringify = Stringify::new(&attribute_storage);
//         widget_tree.apply_visitor(&mut stringify);
//         let output = stringify.finish();

//         eprintln!("---------------");

//         let expected = "
// <for>
//     <iter binding = x, index = 2>
//         test Int(9)
//     <iter binding = x, index = 3>
//         test Int(1)
//     <iter binding = x, index = 4>
//         test Int(2)
//     <iter binding = x, index = 5>
//         test Int(3)
//     <iter binding = x, index = 6>
//         test Int(100)
//     <iter binding = x, index = 7>
//         test Int(101)
//     <iter binding = x, index = 8>
//         test Int(102)";
//         assert_eq!(expected.trim(), output.trim());
//     }

//     #[test]
//     fn eval_for() {
//         let mut list = List::empty();
//         list.push_back(1u32);
//         list.push_back(2u32);
//         list.push_back(3u32);
//         list.push_back(4u32);
//         let mut map = Map::<List<_>>::empty();
//         map.insert("a", list);

//         let tpl = "
//         for x in state.a
//             test x
//                 test x
//         ";
//         let (blueprint, globals) = Document::new(tpl).compile().unwrap();
//         let mut tree = WidgetTree::empty();
//         let mut attribute_storage = AttributeStorage::empty();
//         let mut floating_widgets = FloatingWidgets::empty();
//         let mut components = Components::new();
//         let mut dirty_widgets = DirtyWidgets::empty();
//         let factory = setup_test_factory();
//         let mut component_reg = ComponentRegistry::new();
//         let mut states = States::new();
//         let state_id = states.insert(Box::new(map));
//         let mut scope = Scope::new();
//         scope.insert_state(state_id);
//         let mut ctx = EvalContext::new(
//             &globals,
//             &factory,
//             &mut scope,
//             &mut states,
//             &mut component_reg,
//             &mut attribute_storage,
//             &mut floating_widgets,
//             &mut components,
//             &mut dirty_widgets,
//         );
//         eval_blueprint(&blueprint, &mut ctx, root_node(), &mut tree).unwrap();

//         let mut stringify = Stringify::new(&attribute_storage);
//         tree.apply_visitor(&mut stringify);
//         let output = stringify.finish();

//         let expected = "
// <for>
//     <iter binding = x, index = 0>
//         test Int(1)
//             test Int(1)
//     <iter binding = x, index = 1>
//         test Int(2)
//             test Int(2)
//     <iter binding = x, index = 2>
//         test Int(3)
//             test Int(3)
//     <iter binding = x, index = 3>
//         test Int(4)
//             test Int(4)
// ";
//         assert_eq!(expected.trim(), output.trim());
//     }
// }
