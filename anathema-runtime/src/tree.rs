use anathema_state::StateId;
use anathema_widgets::components::{AnyComponent, AnyEventCtx, ComponentContext};
use anathema_widgets::{Elements, WidgetId, WidgetKind, WidgetTree};

use crate::events::EventCtx;

pub(crate) trait Tree<'bp> {
    fn with_component<F, V>(
        &mut self,
        widget_id: WidgetId,
        state_id: StateId,
        event_ctx: &mut EventCtx<'_, '_, 'bp>,
        f: F,
    ) -> Option<V>
    where
        F: FnOnce(&mut dyn AnyComponent, AnyEventCtx<'_, '_, '_>) -> V;
}

impl<'bp> Tree<'bp> for WidgetTree<'bp> {
    fn with_component<F, V>(
        &mut self,
        widget_id: WidgetId,
        state_id: StateId,
        event_ctx: &mut EventCtx<'_, '_, 'bp>,
        f: F,
    ) -> Option<V>
    where
        F: FnOnce(&mut dyn AnyComponent, AnyEventCtx<'_, '_, '_>) -> V,
    {
        self.with_value_mut(widget_id, |path, widget, tree| {
            panic!()
            // let WidgetKind::Component(component) = &mut widget.kind else { return None };
            // let (node, values) = tree.get_node_by_path(path)?;

            // event_ctx
            //     .attribute_storage
            //     .with_mut(component.widget_id, |attributes, attribute_storage| {
            //         let elements = Elements::new(node.children(), values, attribute_storage, event_ctx.dirty_widgets);

            //         let state = event_ctx.states.get_mut(state_id);
            //         let component_ctx = ComponentContext::new(
            //             state_id,
            //             component.parent,
            //             component.assoc_functions,
            //             event_ctx.assoc_events,
            //             event_ctx.focus_queue,
            //             attributes,
            //         );

            //         let event_ctx = AnyEventCtx {
            //             state,
            //             elements,
            //             context: event_ctx.context,
            //             component_ctx,
            //         };

            //         f(&mut *component.dyn_component, event_ctx)
            //     })
        })
    }
}
