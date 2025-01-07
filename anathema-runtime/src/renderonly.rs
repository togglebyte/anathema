use anathema_backend::{Backend, WidgetCycle};
use anathema_state::{Changes, FutureValues, States};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{
    eval_blueprint, AttributeStorage, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, Scope,
    WidgetTree,
};

pub use crate::error::Result;

pub struct OneShot<T> {
    pub(super) factory: Factory,
    pub(super) blueprint: Blueprint,
    pub(super) globals: Globals,
    pub(super) document: Document,
    pub(super) glyph_map: GlyphMap,
    pub(super) backend: T,
    pub(super) viewport: Viewport,
    pub(super) constraints: Constraints,
    pub(super) components: Components,
    pub(super) dirty_widgets: DirtyWidgets,
    pub(super) changes: Changes,
    pub(super) future_values: FutureValues,
    pub(super) component_registry: ComponentRegistry,
    pub(super) floating_widgets: FloatingWidgets,
    pub(super) changelist: ChangeList,
}

impl<T> OneShot<T>
where
    T: Backend,
{
    pub fn run(mut self) -> Result<()> {
        panic!()
        // let (blueprint, globals) = self.document.compile()?;
        // let mut tree = WidgetTree::empty();
        // let mut attribute_storage = AttributeStorage::empty();
        // let mut scope = Scope::new();
        // let mut states = States::new();

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

        // let mut view = tree.view_mut();
        // let res = eval_blueprint(&blueprint, &mut ctx, root_node(), &mut view);

        // let mut cycle = WidgetCycle::new(&mut self.backend, &mut tree, self.constraints);
        // cycle.run(&mut ctx);

        // self.backend.render(&mut self.glyph_map);
        // self.backend.clear();

        // Ok(())
    }
}
