use anathema_backend::test::TestBackend;
use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_state::{State, StateId, States, Value};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::Document;
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, AttributeStorage, Elements, EvalContext, Factory, FloatingWidgets, Query, Scope, ValueStack,
    WidgetKind, WidgetRenderer as _, WidgetTree,
};

use crate::register_default_widgets;

pub struct TestRunner {
    states: States,
    components: ComponentRegistry,
    factory: Factory,
    backend: TestBackend,
    blueprint: Blueprint,
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
        border [border-style: 'thick']
            expand
                @main
        ";
        let mut doc = Document::new(root);
        let main = doc.add_component("main", src);
        components.add_component(main.into(), (), ());

        let blueprint = doc.compile().unwrap().remove(0);

        Self {
            factory,
            backend: TestBackend::new(size),
            states,
            components,
            blueprint,
        }
    }

    pub fn instance(&mut self) -> TestInstance<'_> {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let viewport = Viewport::new(self.backend.surface.size());

        let mut scope = Scope::new();
        scope.insert_state(StateId::ZERO);
        let mut value_store = ValueStack::empty();
        let mut ctx = EvalContext::new(
            &self.factory,
            &mut scope,
            &mut self.states,
            &mut self.components,
            &mut value_store,
            &mut attribute_storage,
            &mut floating_widgets,
        );

        let path = Default::default();
        eval_blueprint(&self.blueprint, &mut ctx, &path, &mut tree);

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

        let mut filter = LayoutFilter::new(true);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(self.text.new_session(), &self.attribute_storage, &self.viewport);
            layout_widget(widget, children, values, constraints, &mut layout_ctx, true);

            // Position
            position_widget(widget, children, values, attribute_storage, true);

            // Paint
            self.backend.paint(
                widget,
                children,
                values,
                &mut self.text.new_session(),
                attribute_storage,
                true,
            );
        });

        // Paint floating widgets
        for widget_id in self.floating_widgets.iter() {
            self.tree.with_nodes_and_values(*widget_id, |widget, children, values| {
                let WidgetKind::Element(el) = widget else { unreachable!("this is always a floating widget") };
                let mut layout_ctx = LayoutCtx::new(self.text.new_session(), &self.attribute_storage, &self.viewport);

                layout_widget(el, children, values, constraints, &mut layout_ctx, true);

                // Position
                position_widget(el, children, values, attribute_storage, true);

                // Paint
                self.backend.paint(
                    el,
                    children,
                    values,
                    &mut self.text.new_session(),
                    attribute_storage,
                    true,
                );
            });
        }

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
        F: FnMut(Query<'_, '_, '_, '_, TestState>),
    {
        // path [0, 0, 0] points to:
        // border [0]
        //     expand [0, 0]
        //          @main [0, 0, 0] <- this one
        let path = [0, 0, 0].into();

        let Some((node, values)) = self.tree.get_node_by_path(&path) else { return self };
        let Some(state) = self.states.get_mut(StateId::ZERO) else { return self };
        let state = state.to_any_mut().downcast_mut::<TestState>();
        let mut widgets = Elements::new(node.children(), values, &mut self.attribute_storage);
        let query = widgets.query(state);
        f(query);
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
