use anathema_state::Change;
use anathema_store::tree::new_node_path;
use anathema_templates::blueprints::Blueprint;

use super::WidgetKind;
use crate::error::{Error, Result};
use crate::expressions::eval_collection;
use crate::nodes::EvalContext;
use crate::scope::Scope;
use crate::values::{Collection, ValueId};
use crate::{eval_blueprint, Value, WidgetTree};

pub(super) const LOOP_INDEX: &str = "loop";

#[derive(Debug)]
pub struct For<'bp> {
    pub(super) binding: &'bp str,
    pub(super) collection: Value<'bp, Collection<'bp>>,
    pub(super) body: &'bp [Blueprint],
}

impl<'bp> For<'bp> {
    pub(super) fn scope_value(&self, scope: &mut Scope<'bp>, index: usize) {
        self.collection.scope(scope, self.binding, index)
    }

    pub(crate) fn collection(&self) -> &Collection<'_> {
        self.collection.inner()
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut EvalContext<'_, '_, 'bp>,
        change: &Change,
        value_id: ValueId,
        path: &[u16],
        tree: &mut WidgetTree<'bp>,
    ) -> Result<()> {
        match change {
            Change::Inserted(index, value) => {
                // 1. Declare insert path
                // 2. Create new iteration
                // 3. Insert new iteration
                // 4. Update index of all subsequent iterations
                // 5. Scope new value
                // 6. Eval body

                ctx.scope.push();
                ctx.scope.scope_pending(self.binding, *value);

                let insert_at = new_node_path(path, *index as u16);
                let iter_id = tree
                    .insert(&insert_at)
                    .commit_at(WidgetKind::Iteration(Iteration {
                        loop_index: anathema_state::Value::new(*index as i64),
                        binding: self.binding,
                    }))
                    .unwrap(); // TODO unwrap

                // Bump the index for every subsequent sibling of the newly inserted node
                tree.children_after(&insert_at, |node, values| {
                    let iter_widget = values.get_mut(node.value());
                    let Some((_, WidgetKind::Iteration(iter))) = iter_widget else { unreachable!() };
                    *iter.loop_index.to_mut() += 1;
                });

                tree.with_value_mut(iter_id, |parent, iter_widget, tree| {
                    // NOTE
                    // The value has to be scoped to the current binding and not
                    // the iteration, since the collection might've changed more than once
                    // and differ from what's represented by the tree.
                    //
                    // E.g
                    // Two inserts at 0 would result in scoping the same value twice:
                    // the current values in the collection at position 0.
                    //
                    // If the list starts out with ["a"]
                    // The tree will contain a ValueRef -> "a".
                    //
                    // If two values are added to the list:
                    // list.insert(0, "b");
                    // list.insert(0, "c");
                    //
                    // The change output will be Change::Insert(0, "b")
                    // The change output will be Change::Insert(0, "c")
                    //
                    // However the list is ["c", "b", "a"] before the first
                    // change is applied, which would lead to scoping `"c" to `0`
                    // twice.
                    let WidgetKind::Iteration(iter) = iter_widget else { unreachable!() };
                    ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

                    for bp in self.body {
                        eval_blueprint(bp, ctx, parent, tree)?;
                    }

                    Ok(())
                })?;

                ctx.scope.pop();
            }
            Change::Removed(index) => {
                let child_to_remove = new_node_path(path, *index as u16);
                tree.remove(&child_to_remove);
            }
            Change::Dropped => {
                tree.remove_children(path);

                // TODO unwrap, ewww
                self.collection = eval_collection(
                    self.collection.expr.unwrap(),
                    ctx.globals,
                    ctx.scope,
                    ctx.states,
                    value_id,
                );

                for index in 0..self.collection.count() {
                    self.scope_value(ctx.scope, index);
                    ctx.scope.push();

                    let iter_id = tree
                        .insert(path)
                        .commit_child(WidgetKind::Iteration(Iteration {
                            loop_index: anathema_state::Value::new(index as i64),
                            binding: self.binding,
                        }))
                        .ok_or(Error::TreeTransactionFailed)?;

                    // Scope the iteration value
                    tree.with_value_mut(iter_id, |parent, widget, tree| -> Result<()> {
                        let WidgetKind::Iteration(iter) = widget else { unreachable!() };
                        ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

                        for bp in self.body {
                            eval_blueprint(bp, ctx, parent, tree)?;
                        }

                        Ok(())
                    })?;

                    ctx.scope.pop();
                }
            }
            Change::Changed => {
                // TODO implement this as an optimisation once the runtime is done.
                //      Use this to flag the element as needs-layout.
                //      Every element that needs layout should apply
                //      layout using cached constraints and size.
                //      If the size changes, then this has to be propagate
                //      throughout the widget tree
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

#[cfg(test)]
mod test {
    use anathema_state::{drain_changes, Changes, List, Map, StateId, States};
    use anathema_store::tree::root_node;
    use anathema_templates::Document;

    use super::*;
    use crate::components::ComponentRegistry;
    use crate::nodes::stringify::Stringify;
    use crate::nodes::{eval_blueprint, update_tree};
    use crate::testing::setup_test_factory;
    use crate::{AttributeStorage, Components, FloatingWidgets};

    #[test]
    fn loop_remove() {
        let mut list = List::empty();
        list.push_back(1u32);
        list.push_back(2u32);
        list.push_back(3u32);

        let mut map = Map::<List<_>>::empty();
        map.insert("a", list);

        let tpl = "
        for x in a
            test x
            // test loop
        ";

        let (blueprint, globals) = Document::new(tpl).compile().unwrap();
        let mut widget_tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let factory = setup_test_factory();
        let mut component_registry = ComponentRegistry::new();
        let mut components = Components::new();
        let mut states = States::new();
        let state_id = states.insert(Box::new(map));
        let mut scope = Scope::new();
        scope.insert_state(state_id);
        let mut ctx = EvalContext::new(
            &globals,
            &factory,
            &mut scope,
            &mut states,
            &mut component_registry,
            &mut attribute_storage,
            &mut floating_widgets,
            &mut components,
        );

        eval_blueprint(&blueprint, &mut ctx, &[], &mut widget_tree).unwrap();

        let mut stringify = Stringify::new(&attribute_storage);
        widget_tree.apply_visitor(&mut stringify);
        let output = stringify.finish();

        println!("{output}");

        {
            let map = states.get_mut(StateId::ZERO).unwrap();
            let map = map
                .to_any_mut()
                .downcast_mut::<anathema_state::Value<Map<List<u32>>>>()
                .unwrap();
            let mut map = map.to_mut();
            let list = map.get_mut("a").unwrap();
            list.insert(0, 9); // 9, 1, 2, 3,
            list.insert(0, 10); // 10, 9, 1, 2, 3
            list.push_back(100); // 10, 9, 1, 2, 3, 100
            list.push_back(101); // 10, 9, 1, 2, 3, 100, 101
            list.push_back(102); // 10, 9, 1, 2, 3, 100, 101, 102
            list.insert(0, 8); // 8, 10, 9,  1, 2, 3, 100, 101, 102
            list.remove(0); // 10, 9, 1, 2, 3, 100, 101, 102
            list.remove(0); // 9, 1, 2, 3, 100, 101, 102
        }

        eprintln!("---------------");
        let mut local_changes = Changes::empty();
        drain_changes(&mut local_changes);
        local_changes.drain().rev().for_each(|(subs, change)| {
            subs.with(|sub| {
                eprintln!("- apply change: {change:?}");
                let mut scope = Scope::with_capacity(10);
                let widget_path = widget_tree.path(sub);
                update_tree(
                    &globals,
                    &factory,
                    &mut scope,
                    &mut states,
                    &mut component_registry,
                    &change,
                    sub,
                    &widget_path,
                    &mut widget_tree,
                    &mut attribute_storage,
                    &mut floating_widgets,
                    &mut components,
                );
            });
        });

        let mut scope = Scope::new();
        scope.insert_state(state_id);

        let mut stringify = Stringify::new(&attribute_storage);
        widget_tree.apply_visitor(&mut stringify);
        let output = stringify.finish();

        eprintln!("---------------");

        let expected = "
<for>
    <iter binding = x, index = 2>
        test Int(9)
    <iter binding = x, index = 3>
        test Int(1)
    <iter binding = x, index = 4>
        test Int(2)
    <iter binding = x, index = 5>
        test Int(3)
    <iter binding = x, index = 6>
        test Int(100)
    <iter binding = x, index = 7>
        test Int(101)
    <iter binding = x, index = 8>
        test Int(102)";
        assert_eq!(expected.trim(), output.trim());
    }

    #[test]
    fn eval_for() {
        let mut list = List::empty();
        list.push_back(1u32);
        list.push_back(2u32);
        list.push_back(3u32);
        list.push_back(4u32);
        let mut map = Map::<List<_>>::empty();
        map.insert("a", list);

        let tpl = "
        for x in a
            test x
                test x
        ";
        let (blueprint, globals) = Document::new(tpl).compile().unwrap();
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let mut components = Components::new();
        let factory = setup_test_factory();
        let mut component_reg = ComponentRegistry::new();
        let mut states = States::new();
        let state_id = states.insert(Box::new(map));
        let mut scope = Scope::new();
        scope.insert_state(state_id);
        let mut ctx = EvalContext::new(
            &globals,
            &factory,
            &mut scope,
            &mut states,
            &mut component_reg,
            &mut attribute_storage,
            &mut floating_widgets,
            &mut components,
        );
        eval_blueprint(&blueprint, &mut ctx, root_node(), &mut tree).unwrap();

        let mut stringify = Stringify::new(&attribute_storage);
        tree.apply_visitor(&mut stringify);
        let output = stringify.finish();

        let expected = "
<for>
    <iter binding = x, index = 0>
        test Int(1)
            test Int(1)
    <iter binding = x, index = 1>
        test Int(2)
            test Int(2)
    <iter binding = x, index = 2>
        test Int(3)
            test Int(3)
    <iter binding = x, index = 3>
        test Int(4)
            test Int(4)
";
        assert_eq!(expected.trim(), output.trim());
    }
}
