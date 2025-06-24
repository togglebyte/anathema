use anathema_state::{Change, Subscriber};

use super::WidgetContainer;
use crate::WidgetKind;
use crate::error::Result;
use crate::layout::LayoutCtx;
use crate::widget::WidgetTreeView;

pub fn update_widget<'bp>(
    widget: &mut WidgetContainer<'bp>,
    value_id: Subscriber,
    change: &Change,
    tree: WidgetTreeView<'_, 'bp>,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Result<()> {
    let attribute_storage = &mut ctx.attribute_storage;
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
        WidgetKind::For(for_loop) => for_loop.update(change, tree, ctx)?,
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(controlflow) => controlflow.update(change, value_id.index().into(), attribute_storage),
        WidgetKind::ControlFlowContainer(_) => unreachable!("control flow containers have no values"),
        WidgetKind::Component(_) => (),
        WidgetKind::Slot => todo!(),
    }

    Ok(())
}
