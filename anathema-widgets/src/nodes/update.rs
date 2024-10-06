use anathema_state::Change;
use anathema_store::tree::PathFinder;

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
use crate::error::Result;
use crate::values::ValueId;
use crate::{Scope, WidgetKind, WidgetTree};

struct UpdateTree<'a, 'b, 'bp> {
    change: &'a Change,
    value_id: ValueId,
    ctx: EvalContext<'a, 'b, 'bp>,
}

impl<'a, 'b, 'bp> PathFinder<WidgetKind<'bp>> for UpdateTree<'a, 'b, 'bp> {
    type Output = Result<()>;

    fn apply(&mut self, node: &mut WidgetKind<'bp>, path: &[u16], tree: &mut WidgetTree<'bp>) -> Self::Output {
        scope_value(node, self.ctx.scope, &[]);
        update_widget(node, &mut self.ctx, self.value_id, self.change, path, tree)?;
        Ok(())
    }

    fn parent(&mut self, parent: &mut WidgetKind<'bp>, children: &[u16]) {
        scope_value(parent, self.ctx.scope, children);
    }
}

/// Scan the widget tree using the node path.
/// Build up the scope from the parent nodes.
pub fn update_tree<'bp>(
    change: &Change,
    value_id: ValueId,
    path: &[u16],
    tree: &mut WidgetTree<'bp>,
    ctx: EvalContext<'_, '_, 'bp>,
) {
    let update = UpdateTree { change, value_id, ctx };
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
                ctx.attribute_storage
                    .with_mut(value_id.key(), |attributes, attribute_storage| {
                        if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
                            value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states, attribute_storage);
                        }
                    });
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
        WidgetKind::Component(component) => {
            if let Change::Dropped = change {
                ctx.attribute_storage
                    .with_mut(component.widget_id, |attributes, component_attributes| {
                        if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
                            value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states, component_attributes);
                        }
                    });
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
            scope.scope_component_attributes(component.widget_id);

            // Insert internal state
            let state_id = component.state_id();
            scope.insert_state(state_id);
        }
        WidgetKind::ControlFlow(_) | WidgetKind::Element(Element { .. }) | WidgetKind::If(_) | WidgetKind::Else(_) => {}
    }
}
