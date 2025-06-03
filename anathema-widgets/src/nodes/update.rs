use anathema_state::{Change, Subscriber};
use anathema_value_resolver::AttributeStorage;

use super::WidgetContainer;
use crate::WidgetKind;
use crate::error::Result;
use crate::widget::WidgetTreeView;

pub fn update_widget<'bp>(
    widget: &mut WidgetContainer<'bp>,
    value_id: Subscriber,
    change: &Change,
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
        WidgetKind::For(for_loop) => for_loop.update(change, tree, attribute_storage)?,
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(controlflow) => controlflow.update(change, value_id.index().into(), attribute_storage),
        WidgetKind::ControlFlowContainer(_) => unreachable!("control flow containers have no values"),
        WidgetKind::Component(_) => (),
        WidgetKind::Slot => todo!(),
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
