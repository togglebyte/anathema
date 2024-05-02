use anathema_state::States;
use anathema_store::tree::{NodePath, PathFinder};

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
use super::update::scope_value;
use crate::components::ComponentRegistry;
use crate::expressions::{eval, eval_collection};
use crate::values::{Collection, ValueId, ValueStack};
use crate::widget::FloatingWidgets;
use crate::{AttributeStorage, Factory, Scope, WidgetKind, WidgetTree};

struct ResolveFutureValues<'a, 'b, 'bp> {
    value_id: ValueId,
    factory: &'a Factory,
    scope: &'b mut Scope<'bp>,
    value_store: &'b mut ValueStack<'bp>,
    states: &'b mut States,
    components: &'b mut ComponentRegistry,
    attribute_storage: &'b mut AttributeStorage<'bp>,
    floating_widgets: &'b mut FloatingWidgets,
}

impl<'a, 'b, 'bp> PathFinder<WidgetKind<'bp>> for ResolveFutureValues<'a, 'b, 'bp> {
    fn apply(&mut self, node: &mut WidgetKind<'bp>, path: &NodePath, tree: &mut WidgetTree<'bp>) {
        scope_value(node, self.scope, &[]);
        let mut ctx = EvalContext {
            factory: self.factory,
            scope: self.scope,
            value_store: self.value_store,
            states: self.states,
            components: self.components,
            attribute_storage: self.attribute_storage,
            floating_widgets: self.floating_widgets,
        };
        try_resolve_value(node, &mut ctx, self.value_id, path, tree);
    }

    fn parent(&mut self, parent: &WidgetKind<'bp>, children: &[u16]) {
        scope_value(parent, self.scope, children);
    }
}

pub fn try_resolve_future_values<'bp>(
    factory: &Factory,
    scope: &mut Scope<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    value_store: &mut ValueStack<'bp>,
    value_id: ValueId,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    let res = ResolveFutureValues {
        value_id,
        factory,
        scope,
        value_store,
        states,
        components,
        attribute_storage,
        floating_widgets,
    };

    tree.apply_path_finder(path, res);
}

fn try_resolve_value<'bp>(
    widget: &mut WidgetKind<'bp>,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    value_id: ValueId,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
) {
    match widget {
        WidgetKind::Element(Element { ident: _, container }) => {
            let Some(val) = ctx.attribute_storage.get_mut(container.id).get_mut(value_id.index()) else {
                return;
            };
            let value = eval(val.expr, ctx.scope, ctx.states, value_id);
            *val = value;
        }
        WidgetKind::For(for_loop) => {
            // 1. Assign a new collection
            // 2. Remove the current children
            // 3. Build up new children

            for_loop.collection = eval_collection(for_loop.collection.expr, ctx.scope, ctx.states, value_id);
            tree.remove_children(path);
            let collection = &for_loop.collection;
            let binding = &for_loop.binding;
            let body = for_loop.body;
            let parent = path;

            for index in 0..collection.len() {
                ctx.scope.push();

                match collection.inner() {
                    Collection::Static(expressions) => {
                        let downgrade = expressions[index].downgrade();
                        ctx.scope.scope_downgrade(binding, downgrade)
                    }
                    Collection::Dyn(value_ref) => {
                        let value = value_ref
                            .as_state()
                            .and_then(|state| state.state_lookup(index.into()))
                            .unwrap(); // TODO: ewwww
                        ctx.scope.scope_pending(binding, value)
                    }
                    Collection::Future => {}
                }

                let iter_id = tree
                    .insert(parent)
                    .commit_child(WidgetKind::Iteration(super::loops::Iteration {
                        loop_index: anathema_state::Value::new(index as i64),
                        binding,
                    }))
                    .unwrap();

                // Scope the iteration value
                tree.with_value(iter_id, |parent, widget, tree| {
                    let WidgetKind::Iteration(iter) = widget else { unreachable!() };
                    ctx.scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());

                    for bp in body {
                        crate::eval_blueprint(bp, ctx, parent, tree);
                    }
                });

                ctx.scope.pop();
            }
        }
        WidgetKind::If(widget) => {
            let expr = widget.cond.expr;
            let value = eval(expr, ctx.scope, ctx.states, value_id);
            widget.cond = value;
        }
        WidgetKind::Else(el) => {
            let Some(val) = &mut el.cond else { return };
            let expr = val.expr;
            *val = eval(expr, ctx.scope, ctx.states, value_id);
        }
        WidgetKind::ControlFlow(_) => unreachable!(),
        WidgetKind::Iteration(_) => unreachable!(),
        WidgetKind::Component(component) => {
            let Some(state) = &mut component.external_state else { return };
            for ((_, i), v) in state.iter_mut() {
                if *i == value_id.index() {
                    *v = eval(v.expr, ctx.scope, ctx.states, value_id);
                }
            }
        }
    }
}
