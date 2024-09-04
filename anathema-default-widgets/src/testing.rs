use anathema_backend::test::TestBackend;
use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::{Pos, Size};
use anathema_state::{State, StateId, States, Value};
use anathema_store::tree::AsNodePath;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals, ToSourceKind};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, AttributeStorage, Components, Elements, EvalContext, Factory, FloatingWidgets, Scope, WidgetKind,
    WidgetRenderer as _, WidgetTree,
};

use crate::register_default_widgets;

pub struct TestRunner {
    states: States,
    component_registry: ComponentRegistry,
    factory: Factory,
    backend: TestBackend,
    blueprint: Blueprint,
    globals: Globals,
}

impl TestRunner {
    pub fn new(src: &str, size: impl Into<Size>) -> Self {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let mut components = ComponentRegistry::new();
        let mut states = States::new();
        states.insert(Box::new(TestState::new()));

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
        components.add_component(main.into(), (), ());

        let (blueprint, globals) = doc.compile().unwrap();

        Self {
            factory,
            backend: TestBackend::new(size),
            states,
            component_registry: components,
            blueprint,
            globals,
        }
    }

    pub fn instance(&mut self) -> TestInstance<'_> {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let mut components = Components::new();
        let viewport = Viewport::new(self.backend.surface.size());

        let mut scope = Scope::new();
        scope.insert_state(StateId::ZERO);
        let mut ctx = EvalContext::new(
            &self.globals,
            &self.factory,
            &mut scope,
            &mut self.states,
            &mut self.component_registry,
            &mut attribute_storage,
            &mut floating_widgets,
            &mut components,
        );

        eval_blueprint(&self.blueprint, &mut ctx, &[], &mut tree).unwrap();

        TestInstance {
            states: &mut self.states,
            backend: &mut self.backend,
            floating_widgets,
            tree,
            attribute_storage,
            text: StringStorage::new(),
            viewport,
        }
    }
}

pub struct TestInstance<'bp> {
    tree: WidgetTree<'bp>,
    attribute_storage: AttributeStorage<'bp>,
    floating_widgets: FloatingWidgets,
    text: StringStorage,
    states: &'bp mut States,
    backend: &'bp mut TestBackend,
    viewport: Viewport,
}

impl TestInstance<'_> {
    pub fn with_state<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut TestState),
    {
        let state = self.states.get_mut(StateId::ZERO).unwrap();
        let state = state.to_any_mut().downcast_mut::<TestState>().unwrap();
        f(state);
        self
    }

    pub fn render_assert(&mut self, expected: &str) -> &mut Self {
        let expected = expected.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        let (width, height) = self.backend.surface.size().into();
        let constraints = Constraints::new(width as usize, height as usize);

        let attribute_storage = &self.attribute_storage;
        let mut string_session = self.text.new_session();

        WidgetCycle::new(
            self.backend,
            &mut self.tree,
            constraints,
            attribute_storage,
            &mut string_session,
            &self.floating_widgets,
            self.viewport,
        )
        .run();

        self.backend.render();

        let actual = std::mem::take(&mut self.backend.output);
        let actual = actual.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        self.text.clear();

        eprintln!("{actual}");

        assert_eq!(actual, expected);
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
        let elements = Elements::new(node.children(), values, &mut self.attribute_storage);
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
