use anathema_state::{Change, Subscriber};

use super::WidgetContainer;
use crate::error::Result;
use crate::layout::LayoutCtx;
use crate::widget::WidgetTreeView;
use crate::{DirtyWidgets, WidgetKind};

pub fn update_widget<'bp>(
    widget: &mut WidgetContainer<'bp>,
    value_id: Subscriber,
    change: &Change,
    tree: WidgetTreeView<'_, 'bp>,
    ctx: &mut LayoutCtx<'_, 'bp>,
    dirty_widgets: &mut DirtyWidgets,
) -> Result<()> {
    match &mut widget.kind {
        WidgetKind::Element(element) => {
            ctx.attribute_storage.with_mut(element.id(), |attributes, storage| {
                let Some(value) = attributes.get_mut_with_index(value_id.index()) else { return };
                value.reload(storage);
                dirty_widgets.push(element.id());
            });

            if let Change::Dropped = change {
                // TODO: Is there anything that needs to be done here given that the value will
            }
        }
        WidgetKind::For(for_loop) => for_loop.update(change, tree, ctx, widget.parent_widget, dirty_widgets)?,
        WidgetKind::With(with) => with.update(change, tree, ctx.attribute_storage)?,
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(controlflow) => controlflow.update(
            change,
            value_id.index().into(),
            tree,
            ctx,
            widget.parent_widget,
            dirty_widgets,
        ),
        WidgetKind::ControlFlowContainer(_) => unreachable!("control flow containers have no values"),
        WidgetKind::Component(_) => (),
        WidgetKind::Slot => todo!(),
    }

    Ok(())
}
