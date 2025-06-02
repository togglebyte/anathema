use anathema::{Changes, drain_changes};
use anathema_backend::test::TestBackend;
use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{OldState, StateId, States, Value};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals, ToSourceKind};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{
    AttributeStorage, Components, DirtyWidgets, Elements, EvalContext, Factory, FloatingWidgets, GlyphMap, Scope,
    WidgetRenderer as _, WidgetTree, update_tree,
};

use crate::register_default_widgets;

pub struct TestRunner {
    states: States,
    component_registry: ComponentRegistry,
    factory: Factory,
    backend: TestBackend,
    blueprint: Blueprint,
    globals: Globals,
    components: Components,
    dirty_widgets: DirtyWidgets,
}

impl TestRunner {
    pub fn new(src: &str, size: impl Into<Size>) -> Self {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let mut component_registry = ComponentRegistry::new();
        let states = States::new();

        // Add two to both dimensions to compensate
        // for the border size that we inject here.
        let mut size = size.into();
        size.width += 2;
        size.height += 2;

        let root = "
        border [border_style: 'thick']
            expand
                @main
        ";
        let mut doc = Document::new(root);
        let main = doc.add_component("main", src.to_template()).unwrap();
        component_registry.add_component(main.into(), (), TestState::new());

        let (blueprint, globals) = doc.compile().unwrap();

        Self {
            factory,
            backend: TestBackend::new(size),
            states,
            component_registry,
            blueprint,
            globals,
            components: Components::new(),
            dirty_widgets: DirtyWidgets::empty(),
        }
    }

    pub fn instance(&mut self) -> TestInstance<'_> {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let viewport = Viewport::new(self.backend.surface.size());

        let mut scope = Scope::new();
        let mut ctx = EvalContext::new(
            &self.globals,
            &self.factory,
            &mut scope,
            &mut self.states,
            &mut self.component_registry,
            &mut attribute_storage,
            &mut floating_widgets,
            &mut self.components,
            &mut self.dirty_widgets,
        );

        eval_blueprint(&self.blueprint, &mut ctx, &[], &mut tree).unwrap();

        TestInstance {
            states: &mut self.states,
            backend: &mut self.backend,
            globals: &self.globals,
            floating_widgets,
            tree,
            attribute_storage,
            viewport,
            factory: &self.factory,
            component_registry: &mut self.component_registry,
            components: &mut self.components,
            dirty_widgets: &mut self.dirty_widgets,
            changes: Changes::empty(),
            glyph_map: GlyphMap::empty(),
        }
    }
}

pub struct TestInstance<'bp> {
    tree: WidgetTree<'bp>,
    attribute_storage: AttributeStorage<'bp>,
    floating_widgets: FloatingWidgets,
    states: &'bp mut States,
    globals: &'bp Globals,
    backend: &'bp mut TestBackend,
    viewport: Viewport,
    factory: &'bp Factory,
    component_registry: &'bp mut ComponentRegistry,
    components: &'bp mut Components,
    dirty_widgets: &'bp mut DirtyWidgets,
    changes: Changes,
    glyph_map: GlyphMap,
}

impl TestInstance<'_> {
    pub fn with_state<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut TestState),
    {
        let state = self.states.get_mut(StateId::ZERO).unwrap();
        let state = state.to_any_mut().downcast_mut::<TestState>().unwrap();
        f(state);

        let mut scope = Scope::new();
        drain_changes(&mut self.changes);
        self.changes.iter().for_each(|(sub, change)| {
            sub.iter().for_each(|sub| {
                let Some(path): Option<Box<_>> = self.tree.try_path_ref(sub).map(Into::into) else { return };

                let ctx = EvalContext::new(
                    self.globals,
                    self.factory,
                    &mut scope,
                    self.states,
                    self.component_registry,
                    &mut self.attribute_storage,
                    &mut self.floating_widgets,
                    self.components,
                    self.dirty_widgets,
                );

                update_tree(change, sub, &path, &mut self.tree, ctx);
            })
        });

        self
    }

    pub fn render_assert(&mut self, expected: &str) -> &mut Self {
        let expected = expected.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        let (width, height) = self.backend.surface.size().into();
        let constraints = Constraints::new(width as usize, height as usize);

        let attribute_storage = &self.attribute_storage;

        WidgetCycle::new(
            self.backend,
            &mut self.tree,
            &mut self.glyph_map,
            constraints,
            attribute_storage,
            &self.floating_widgets,
            self.viewport,
        )
        .run();

        self.backend.render(&mut self.glyph_map);

        let actual = std::mem::take(&mut self.backend.output);
        let actual = actual.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        assert_eq!(expected, actual, "\nExpected:\n{expected}\nGot:\n{actual}");
        self
    }

    pub(crate) fn with_widget<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(Elements<'_, '_>),
    {
        // path [0, 0, 0] points to:
        // border [0]
        //     expand [0, 0]
        //          @main [0, 0, 0] <- this one
        let path = [0, 0, 0];

        let Some((node, values)) = self.tree.get_node_by_path(&path) else { return self };
        let elements = Elements::new(
            node.children(),
            values,
            &mut self.attribute_storage,
            &mut self.dirty_widgets,
        );
        f(elements);
        self
    }
}

#[derive(State)]
pub struct TestState {
    pub value: Value<usize>,
    pub offset: Value<i32>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            value: 0.into(),
            offset: 0.into(),
        }
    }
}
