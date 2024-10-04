use anathema_state::{Change, States};
use anathema_store::tree::PathFinder;
use anathema_templates::Globals;

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
use crate::components::{ComponentAttributeCollection, ComponentRegistry};
use crate::error::Result;
use crate::values::ValueId;
use crate::widget::{Components, FloatingWidgets};
use crate::{AttributeStorage, DirtyWidgets, Factory, Scope, WidgetKind, WidgetNeeds, WidgetTree};

struct UpdateTree<'a, 'b, 'bp> {
    globals: &'bp Globals,
    value_id: ValueId,
    change: &'a Change,
    factory: &'a Factory,
    scope: &'b mut Scope<'bp>,
    states: &'b mut States,
    component_attributes: &'b mut ComponentAttributeCollection<'bp>,
    component_registry: &'b mut ComponentRegistry,
    attribute_storage: &'b mut AttributeStorage<'bp>,
    floating_widgets: &'b mut FloatingWidgets,
    components: &'b mut Components,
    dirty_widgets: &'b mut DirtyWidgets,
}

impl<'a, 'b, 'bp> PathFinder<WidgetKind<'bp>> for UpdateTree<'a, 'b, 'bp> {
    type Output = Result<()>;

    fn apply(&mut self, node: &mut WidgetKind<'bp>, path: &[u16], tree: &mut WidgetTree<'bp>) -> Self::Output {
        scope_value(node, self.scope, &[]);
        let mut ctx = EvalContext::new(
            self.globals,
            self.factory,
            self.scope,
            self.states,
            self.component_attributes,
            self.component_registry,
            self.attribute_storage,
            self.floating_widgets,
            self.components,
            self.dirty_widgets,
        );
        update_widget(node, &mut ctx, self.value_id, self.change, path, tree)?;

        Ok(())
    }

    fn parent(&mut self, parent: &mut WidgetKind<'bp>, children: &[u16]) {
        if let WidgetKind::Element(el) = parent {
            el.container.needs = WidgetNeeds::Layout;
        }
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
    component_attributes: &mut ComponentAttributeCollection<'bp>,
    component_registry: &mut ComponentRegistry,
    change: &Change,
    value_id: ValueId,
    path: &[u16],
    tree: &mut WidgetTree<'bp>,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
    components: &mut Components,
    dirty_widgets: &mut DirtyWidgets,
) {
    let update = UpdateTree {
        globals,
        value_id,
        change,
        factory,
        scope,
        states,
        component_attributes,
        component_registry,
        attribute_storage,
        floating_widgets,
        components,
        dirty_widgets,
    };
    tree.apply_path_finder(path, update);
}

fn update_widget<'bp>(
    widget: &mut WidgetKind<'bp>,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    value_id: ValueId,
    change: &Change,
    path: &[u16],
    tree: &mut WidgetTree<'bp>,
) -> Result<()> {
    // Tell all widgets they need layout

    match widget {
        WidgetKind::Element(..) => {
            // Reflow of the layout will be triggered by the runtime and not in this step

            // Any dropped dyn value should register for future updates.
            // This is done by reloading the value, making it empty
            if let Change::Dropped = change {
                let attributes = ctx.attribute_storage.get_mut(value_id.key());
                if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
                    value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states, ctx.component_attributes);
                }
            }
        }
        WidgetKind::For(for_loop) => for_loop.update(ctx, change, value_id, path, tree)?,
        WidgetKind::Iteration(_) => todo!(),
        // but rather the ControlFlow is managing
        // these in the layout process instead, as
        // the ControlFlow has access to all the
        // branches.
        WidgetKind::ControlFlow(_) => unreachable!("update is never called on ControlFlow, only the children"),
        WidgetKind::If(_) | WidgetKind::Else(_) => (), // If / Else are not updated by themselves
        WidgetKind::Component(_) => {
            if let Change::Dropped = change {
                ctx.components.remove(path);
            }
        }
    }

    Ok(())
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
            scope.scope_component_attributes(component.component_id);

            // Insert internal state
            let state_id = component.state_id();
            scope.insert_state(state_id);
        }
        WidgetKind::ControlFlow(_) | WidgetKind::Element(Element { .. }) | WidgetKind::If(_) | WidgetKind::Else(_) => {}
    }
}
