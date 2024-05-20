use anathema_state::{Change, States};
use anathema_store::tree::{NodePath, PathFinder};
use anathema_templates::Globals;

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
use crate::components::ComponentRegistry;
use crate::values::ValueId;
use crate::widget::FloatingWidgets;
use crate::{AttributeStorage, Factory, Scope, WidgetKind, WidgetTree};

struct UpdateTree<'a, 'b, 'bp> {
    globals: &'bp Globals,
    value_id: ValueId,
    change: &'a Change,
    factory: &'a Factory,
    scope: &'b mut Scope<'bp>,
    states: &'b mut States,
    components: &'b mut ComponentRegistry,
    attribute_storage: &'b mut AttributeStorage<'bp>,
    floating_widgets: &'b mut FloatingWidgets,
}

impl<'a, 'b, 'bp> PathFinder<WidgetKind<'bp>> for UpdateTree<'a, 'b, 'bp> {
    fn apply(&mut self, node: &mut WidgetKind<'bp>, path: &NodePath, tree: &mut WidgetTree<'bp>) {
        scope_value(node, self.scope, &[]);
        let mut ctx = EvalContext {
            globals: self.globals,
            factory: self.factory,
            scope: self.scope,
            states: self.states,
            components: self.components,
            attribute_storage: self.attribute_storage,
            floating_widgets: self.floating_widgets,
        };
        update_widget(node, &mut ctx, self.value_id, self.change, path, tree);
    }

    fn parent(&mut self, parent: &WidgetKind<'bp>, children: &[u16]) {
        scope_value(parent, self.scope, children);
    }
}

/// Scan the widget tree using the node path.
/// Build up the scope from the parent nodes.
pub fn update_tree<'bp>(
    globals: &'bp Globals,
    factory: &Factory,
    scope: &mut Scope<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    change: &Change,
    value_id: ValueId,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    let update = UpdateTree {
        globals,
        value_id,
        change,
        factory,
        scope,
        states,
        components,
        attribute_storage,
        floating_widgets,
    };
    tree.apply_path_finder(path, update);
}

fn update_widget<'bp>(
    widget: &mut WidgetKind<'bp>,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    value_id: ValueId,
    change: &Change,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
) {
    // Any dropped dyn value should register for future updates.
    // This is done by reloading the value, making it empty
    if let Change::Dropped | Change::Changed = change {
        let attributes = ctx.attribute_storage.get_mut(value_id.key());
        if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
            value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states);
        }
    }

    match widget {
        WidgetKind::Element(..) => {
            // Reflow of the layout will be triggered by the runtime and not in this step
        }
        WidgetKind::For(for_loop) => for_loop.update(ctx, change, value_id, path, tree),
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(_) => unreachable!("update is never called on ControlFlow, only the children"),
        WidgetKind::If(_) | WidgetKind::Else(_) => (), // If / Else are not updated by themselves
        // but rather the ControlFlow is managing
        // these in the layout process instead, as
        // the ControlFlow has access to all the
        // branches.
        WidgetKind::Component(_) => unreachable!("components do not receive updates"),
    }
}

pub(super) fn scope_value<'bp>(widget: &WidgetKind<'bp>, scope: &mut Scope<'bp>, children: &[u16]) {
    match widget {
        WidgetKind::For(for_loop) => match children {
            [next, ..] => {
                let index = *next as usize;
                for_loop.collection.scope(scope, for_loop.binding, index);
            }
            [] => {}
        },
        WidgetKind::Iteration(iter) => {
            scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());
        }
        WidgetKind::Component(component) => {
            if let Some(state) = &component.external_state {
                for ((k, _), v) in state.iter() {
                    let v = v.downgrade();
                    scope.scope_downgrade(k, v);
                }
            }
            // Insert internal state
            if let Some(state_id) = component.state_id() {
                scope.insert_state(state_id);
            }
        }
        WidgetKind::ControlFlow(_) | WidgetKind::Element(Element { .. }) | WidgetKind::If(_) | WidgetKind::Else(_) => {}
    }
}
