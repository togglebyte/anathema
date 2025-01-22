use anathema_state::{Change, Subscriber};
use anathema_store::tree::PathFinder;
use anathema_value_resolver::AttributeStorage;

use super::element::Element;
use super::loops::LOOP_INDEX;
use super::WidgetContainer;
use crate::error::Result;
use crate::widget::WidgetTreeView;
use crate::{WidgetKind, WidgetTree};

// struct UpdateTree<'rt, 'bp> {
//     change: &'a Change,
//     value_id: ValueId,
//     ctx: EvalContext<'rt, 'bp>,
// }

// impl<'a, 'b, 'bp> PathFinder for UpdateTree<'a, 'b, 'bp> {
//     type Input = WidgetContainer<'bp>;
//     type Output = Result<()>;

//     fn apply(&mut self, node: &mut WidgetContainer<'bp>, path: &[u16], tree: &mut WidgetTree<'bp>) -> Self::Output {
//         panic!("old hat, don't use the path finder anymore, use the new ForEach");
//         // scope_value(&node.kind, self.ctx.scope, &[]);
//         // update_widget(&mut node.kind, &mut self.ctx, self.value_id, self.change, path, tree)?;
//         Ok(())
//     }

//     fn parent(&mut self, parent: &mut WidgetContainer<'bp>, sub_path: &[u16]) {
//         panic!("old hat, don't use the path finder anymore, use the new ForEach");
//         // scope_value(&parent.kind, self.ctx.scope, sub_path);
//     }
// }

pub fn update_widget<'bp>(
    widget: &mut WidgetContainer<'bp>,
    // ctx: &mut EvalContext<'_, 'bp>,
    value_id: Subscriber,
    change: &Change,
    path: &[u16],
    tree: WidgetTreeView<'_, 'bp>,
    attribute_storage: &mut AttributeStorage<'bp>,
) -> Result<()> {
    match &mut widget.kind {
        WidgetKind::Element(element) => {
            attribute_storage.with_mut(element.container.id, |attributes, storage| {
                let Some(value) = attributes.get_mut_with_index(value_id.index()) else { return };
                value.reload(storage);
            });

            if let Change::Dropped = change {
                // TODO: figure out why this is still here?
                // ctx.attribute_storage
                //     .with_mut(value_id.key(), |attributes, attribute_storage| {
                //         if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
                //             value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states, attribute_storage);
                //         }
                //     });
            }
        }
        WidgetKind::For(for_loop) => for_loop.update(change, value_id, tree)?,
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(controlflow) => {
            // TODO: is this even needed? - TB 2024-12-21
            // controlflow
            //     .elses
            //     .iter_mut()
            //     .enumerate()
            //     .filter_map(|(id, cf)| cf.cond.as_mut().map(|cond| (id, cond)))
            //     .for_each(|(id, cond)| {
            //         if ValueIndex::from(id) == value_id.index() {
            //         }
            //     });
        }
        WidgetKind::ControlFlowContainer(_) => unreachable!("control flow containers have no values"),
        WidgetKind::Component(component) => {
            if let Change::Dropped = change {
                // ctx.attribute_storage
                //     .with_mut(component.widget_id, |attributes, component_attributes| {
                //         if let Some(value) = attributes.get_mut_with_index(value_id.index()) {
                //             value.reload_val(value_id, ctx.globals, ctx.scope, ctx.states, component_attributes);
                //         }
                //     });
            }
        }
    }

    Ok(())
}

// pub(super) fn scope_value<'bp>(widget: &WidgetKind<'bp>, scope: &mut Scope<'bp>, children: &[u16]) {
//     match widget {
//         WidgetKind::For(for_loop) => match children {
//             [next, ..] => {
//                 panic!("this should not be used, instead this should happen in the LayoutForEach");
//                 // let index = *next as usize;
//                 // for_loop.collection.scope(scope, for_loop.binding, index);
//             }
//             [] => {}
//         },
//         WidgetKind::Iteration(iter) => {
//             scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());
//         }
//         WidgetKind::Component(component) => {
//             scope.scope_component_attributes(component.widget_id);

//             // Insert internal state
//             let state_id = component.state_id();
//             scope.insert_state(state_id);
//         }
//         _ => panic!("let's remove this in favour of the local tree impl"),
//         // WidgetKind::ControlFlow(_) | WidgetKind::Element(Element { .. }) | WidgetKind::If(_) | WidgetKind::Else(_) => {}
//     }
// }
