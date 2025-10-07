use anathema::{Changes, drain_changes};
use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::{Pos, Size};
use anathema_state::{State, StateId, States, Value};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, ToSourceKind, Variables};
use anathema_value_resolver::{AttributeStorage, Attributes, FunctionTable, Scope};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{Constraints, LayoutCtx, Viewport};
use anathema_widgets::paint::{Glyph, paint};
use anathema_widgets::query::{Children, Elements};
use anathema_widgets::{
    Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, Style, WidgetRenderer, WidgetTree, eval_blueprint,
    update_widget,
};

use crate::register_default_widgets;

pub struct TestBackend {
    pub surface: TestSurface,
    pub output: String,
}

impl TestBackend {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            surface: TestSurface::new(size),
            output: String::new(),
        }
    }
}

impl Backend for TestBackend {
    fn size(&self) -> Size {
        self.surface.size
    }

    fn next_event(&mut self, _timeout: std::time::Duration) -> Option<Event> {
        None
    }

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: anathema_widgets::PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        paint(&mut self.surface, glyph_map, widgets, attribute_storage);
    }

    fn render(&mut self, _glyph_map: &mut GlyphMap) {
        self.output = format!("{}", self.surface);
    }

    fn clear(&mut self) {
        self.surface.clear();
    }

    fn finalize(&mut self) {}

    fn resize(&mut self, _new_size: Size, _glyph_map: &mut GlyphMap) {
        todo!()
    }
}

pub struct TestSurface {
    size: Size,
    buffer: Vec<char>,
}

impl TestSurface {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        let buffer_size = size.width * size.height;
        Self {
            buffer: vec![' '; buffer_size as usize],
            size,
        }
    }

    fn clear(&mut self) {
        self.buffer.fill_with(|| ' ');
    }
}

impl WidgetRenderer for TestSurface {
    fn draw_glyph(&mut self, c: Glyph, local_pos: Pos) {
        let y_offset = local_pos.y as usize * self.size.width as usize;
        let x_offset = local_pos.x as usize;
        let index = y_offset + x_offset;

        match c {
            Glyph::Single(c, _width) => self.buffer[index] = c,
            Glyph::Cluster(_glyph_index, _) => todo!(),
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn set_attributes(&mut self, attribs: &Attributes<'_>, local_pos: Pos) {
        let style = Style::from_cell_attribs(attribs);
        self.set_style(style, local_pos);
    }

    // This does nothing for the test backend,
    // which is only used to test layouts
    fn set_style(&mut self, _: Style, _: Pos) {}
}

impl std::fmt::Display for TestSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let idx = y * self.size.width + x;
                write!(f, "{}", self.buffer[idx as usize])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

pub struct TestRunner {
    states: States,
    component_registry: ComponentRegistry,
    factory: Factory,
    backend: TestBackend,
    blueprint: Blueprint,
    variables: Variables,
    components: Components,
    function_table: FunctionTable,
    doc: Document,
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
        component_registry.add_component(main, (), TestState::new());

        let mut variables = Default::default();
        let blueprint = doc.compile(&mut variables).unwrap();

        Self {
            factory,
            backend: TestBackend::new(size),
            states,
            component_registry,
            blueprint,
            variables,
            components: Components::new(),
            function_table: FunctionTable::new(),
            doc,
        }
    }

    pub fn instance(&mut self) -> TestInstance<'_> {
        TestInstance::new(
            &self.blueprint,
            &mut self.states,
            &mut self.backend,
            &self.variables,
            &self.factory,
            &mut self.component_registry,
            &mut self.components,
            &self.function_table,
            &self.doc,
        )
    }
}

pub struct TestInstance<'bp> {
    tree: WidgetTree<'bp>,
    attribute_storage: AttributeStorage<'bp>,
    floating_widgets: FloatingWidgets,
    states: &'bp mut States,
    variables: &'bp Variables,
    backend: &'bp mut TestBackend,
    viewport: Viewport,
    factory: &'bp Factory,
    component_registry: &'bp mut ComponentRegistry,
    components: &'bp mut Components,
    changes: Changes,
    glyph_map: GlyphMap,
    function_table: &'bp FunctionTable,
    doc: &'bp Document,
}

impl<'bp> TestInstance<'bp> {
    fn new(
        blueprint: &'bp Blueprint,
        states: &'bp mut States,
        backend: &'bp mut TestBackend,
        variables: &'bp Variables,
        factory: &'bp Factory,
        component_registry: &'bp mut ComponentRegistry,
        components: &'bp mut Components,
        function_table: &'bp FunctionTable,
        doc: &'bp Document,
    ) -> Self {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let mut viewport = Viewport::new(backend.size());
        let mut glyph_map = GlyphMap::empty();

        let scope = Scope::root();
        let mut ctx = LayoutCtx::new(
            variables,
            factory,
            states,
            &mut attribute_storage,
            components,
            component_registry,
            &mut floating_widgets,
            &mut glyph_map,
            &mut viewport,
            function_table,
            &doc.expressions,
        );

        let mut ctx = ctx.eval_ctx(None, None);
        let mut view = tree.view();

        eval_blueprint(blueprint, &mut ctx, &scope, &[], &mut view).unwrap();

        Self {
            tree,
            attribute_storage,
            floating_widgets,
            states,
            variables,
            backend,
            viewport,
            factory,
            component_registry,
            components,
            changes: Changes::empty(),
            glyph_map,
            function_table,
            doc,
        }
    }

    pub fn with_state<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut TestState),
    {
        let state = self.states.get_mut(StateId::ZERO).unwrap();
        let mut state = state.to_mut_cast::<TestState>();
        f(&mut state);
        drop(state);

        drain_changes(&mut self.changes);

        if self.changes.is_empty() {
            return self;
        }

        let mut tree = self.tree.view();

        let mut ctx = LayoutCtx::new(
            self.variables,
            self.factory,
            self.states,
            &mut self.attribute_storage,
            self.components,
            self.component_registry,
            &mut self.floating_widgets,
            &mut self.glyph_map,
            &mut self.viewport,
            self.function_table,
            &self.doc.expressions,
        );

        self.changes.iter().for_each(|(sub, change)| {
            sub.iter().for_each(|value_id| {
                let widget_id = value_id.key();

                // TODO: review if this is still needed: - TB 2025-08-27
                // if let Some(widget) = tree.get_mut(widget_id) {
                //     if let WidgetKind::Element(element) = &mut widget.kind {
                //         element.invalidate_cache();
                //     }
                // }

                // check that the node hasn't already been removed
                if !tree.contains(widget_id) {
                    return;
                }

                _ = tree
                    .with_value_mut(value_id.key(), |_path, widget, tree| {
                        update_widget(widget, value_id, change, tree, &mut ctx, &mut DirtyWidgets::empty())
                    })
                    .unwrap();
            })
        });

        self
    }

    pub fn render_assert(&mut self, expected: &str) -> &mut Self {
        let expected = expected.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        let (width, height) = self.backend.size().into();
        let constraints = Constraints::new(width as u16, height as u16);

        let mut ctx = LayoutCtx::new(
            self.variables,
            self.factory,
            self.states,
            &mut self.attribute_storage,
            self.components,
            self.component_registry,
            &mut self.floating_widgets,
            &mut self.glyph_map,
            &mut self.viewport,
            self.function_table,
            &self.doc.expressions,
        );

        let mut cycle = WidgetCycle::new(self.backend, self.tree.view(), constraints);
        _ = cycle.run(&mut ctx, true, &mut DirtyWidgets::empty());

        self.backend.render(&mut self.glyph_map);

        let actual = std::mem::take(&mut self.backend.output);
        let actual = actual.trim().lines().map(str::trim).collect::<Vec<_>>().join("\n");

        assert_eq!(expected, actual, "\nExpected:\n{expected}\nGot:\n{actual}");
        self
    }

    pub(crate) fn with_widget<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(Elements<'_, '_, '_>),
    {
        // path [0, 0, 0] points to:
        // border [0]
        //     expand [0, 0]
        //          @main [0, 0, 0] <- this one
        // let path = &[0, 0, 0];

        let tree = self.tree.view();
        let mut dirty_widgets = DirtyWidgets::empty();
        let mut children = Children::new(tree, &mut self.attribute_storage, &mut dirty_widgets);
        let elements = children.elements();
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
