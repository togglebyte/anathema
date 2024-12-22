// -----------------------------------------------------------------------------
//   - Notes on viewstructs -
//   Well, it's not really about viewstructs but here goes:
//   * Split the runtime into different sections with different borrowed values
// -----------------------------------------------------------------------------

use std::time::Duration;

use anathema_backend::Backend;
use anathema_state::States;
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::events::Event;
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::{
    eval_blueprint, AttributeStorage, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, Scope,
    WidgetTree, WidgetTreeView,
};

pub use crate::error::Result;

pub struct Runtime<'bp, B> {
    pub(super) blueprint: &'bp Blueprint,
    pub(super) globals: &'bp Globals,
    pub(super) factory: &'bp Factory,
    pub(super) tree: WidgetTree<'bp>,
    pub(super) attribute_storage: AttributeStorage<'bp>,
    pub(super) backend: B,
    pub(super) component_registry: &'bp mut ComponentRegistry,
    pub(super) components: Components,
    pub(super) document: &'bp mut Document,
    pub(super) floating_widgets: FloatingWidgets,
    pub(super) changelist: ChangeList,
    pub(super) dirty_widgets: DirtyWidgets,
    pub(super) glyph_map: GlyphMap,
}

impl<'bp, B> Runtime<'bp, B>
where
    B: Backend,
{
    pub fn next_frame(&mut self) -> Result<Frame<'_, 'bp, B>> {
        // let mut tree = WidgetTree::empty();
        // let mut view = tree.view_mut();
        // let mut scope = Scope::new();
        // let mut states = States::new();

        let size = self.backend.size();
        let viewport = Viewport::new(size);
        // let mut attribute_storage = AttributeStorage::empty();

        // let res = eval_blueprint(&self.blueprint, &mut ctx, root_node(), &mut view);
        // let inst = RuntimeParlaver {
        //     backend: &mut self.backend,
        //     document: &self.document,
        //     tree,
        //     eval_ctx: ctx,
        // };

        let layout_ctx = LayoutCtx::new(
            self.globals,
            &self.factory,
            &mut self.attribute_storage,
            &mut self.components,
            &mut self.component_registry,
            &mut self.floating_widgets,
            &mut self.changelist,
            &mut self.glyph_map,
            &mut self.dirty_widgets,
            viewport,
            true,
        );

        let inst = Frame {
            backend: &mut self.backend,
            document: self.document,
            tree: self.tree.view_mut(),
            layout_ctx,
        };

        Ok(inst)
    }
}

pub struct Frame<'rt, 'bp, B> {
    backend: &'rt mut B,
    document: &'rt mut Document,
    tree: WidgetTreeView<'rt, 'bp>,
    layout_ctx: LayoutCtx<'rt, 'bp>,
}

impl<B> Frame<'_, '_, B>
where
    B: Backend,
{
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Noop => return,
            Event::Stop => todo!(),
            Event::Blur => todo!(),
            Event::Focus => todo!(),
            Event::Key(key_event) => {}
            Event::Mouse(mouse_event) => {
                // for i in 0..self.eval_ctx.components.len() {
                //     let (widget_id, state_id) = self
                //         .eval_ctx
                //         .components
                //         .get(i)
                //         .expect("components can not change during this call");

                //     // tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_event(ctx, event));
                // }
            }
            Event::Resize(size) => todo!(),
        }
    }

    pub fn tick(&mut self) {
        // let mut ctx = EvalContext::new(
        //     &globals,
        //     &self.factory,
        //     &mut scope,
        //     &mut states,
        //     &mut self.component_registry,
        //     &mut attribute_storage,
        //     &mut self.floating_widgets,
        //     &mut self.changelist,
        //     &mut self.components,
        //     &mut self.dirty_widgets,
        //     &self.viewport,
        //     &mut self.glyph_map,
        //     true,
        // );
    }

    pub fn present(self) {
    }

    fn poll_event(&mut self, poll_timeout: Duration) {
        let Some(event) = self.backend.next_event(poll_timeout) else { return };
        self.event(event);
    }
}
