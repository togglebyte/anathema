use std::marker::PhantomData;

use anathema_geometry::Size;
use anathema_state::{drain_changes, drain_futures, Changes, FutureValues, State, StateId, States};
use anathema_store::tree::NodePath;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, EvalContext, Factory, FloatingWidgets,
    LayoutChildren, Scope, Stringify, Widget, WidgetTree,
};

#[macro_export]
macro_rules! input {
    ($pth:expr) => {
        include_str!(concat!("frames/", $pth, "/in"))
    };
}

#[macro_export]
macro_rules! out {
    ($pth:expr, $frame:expr) => {
        include_str!(concat!("frames/", $pth, "/", $frame))
    };
}

pub struct TestCaseRunner<'bp, S> {
    _p: PhantomData<S>,

    globals: &'bp Globals,
    blueprint: &'bp Blueprint,
    factory: Factory,
    tree: WidgetTree<'bp>,
    attribute_storage: AttributeStorage<'bp>,
    floating_widgets: FloatingWidgets,
    text: StringStorage,
    states: States,
    components: ComponentRegistry,
    future_values: FutureValues,
    changes: Changes,
    viewport: Viewport,
}

impl<'bp, S> TestCaseRunner<'bp, S>
where
    S: 'static + State,
{
    fn exec(&mut self) {
        let mut scope = Scope::new();
        scope.insert_state(StateId::ZERO);
        let mut ctx = EvalContext::new(
            &self.globals,
            &self.factory,
            &mut scope,
            &mut self.states,
            &mut self.components,
            &mut self.attribute_storage,
            &mut self.floating_widgets,
        );

        eval_blueprint(self.blueprint, &mut ctx, &NodePath::root(), &mut self.tree);

        // Non floating widgets
        let mut filter = LayoutFilter::new(true, &self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(self.text.new_session(), &self.attribute_storage, &self.viewport);
            layout_widget(
                widget,
                children,
                values,
                self.viewport.constraints(),
                &mut layout_ctx,
                true,
            );
        });

        // Floating widgets
        let mut filter = LayoutFilter::new(false, &self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(self.text.new_session(), &self.attribute_storage, &self.viewport);
            layout_widget(
                widget,
                children,
                values,
                self.viewport.constraints(),
                &mut layout_ctx,
                true,
            );
        });
    }

    pub fn expect_frame(&mut self, frame: &str) -> &mut Self {
        let mut stringify = Stringify::new(&self.attribute_storage);
        self.tree.apply_visitor(&mut stringify);
        let output = stringify.finish();

        eprintln!("frame: \n{}\n--------", frame.trim());
        eprintln!("output: \n{}", output.trim());
        assert_eq!(frame.trim(), output.trim());
        self
    }

    fn apply_futures(&mut self) {
        let mut scope = Scope::with_capacity(10);
        drain_futures(&mut self.future_values);
        self.future_values.drain().for_each(|sub| {
            scope.clear();
            scope.insert_state(StateId::ZERO);
            let path = self.tree.path(sub).clone();

            try_resolve_future_values(
                &self.globals,
                &self.factory,
                &mut scope,
                &mut self.states,
                &mut self.components,
                sub,
                &path,
                &mut self.tree,
                &mut self.attribute_storage,
                &mut self.floating_widgets,
            );
        });
    }

    fn update_tree(&mut self) {
        let mut scope = Scope::with_capacity(10);
        drain_changes(&mut self.changes);
        self.changes.drain().rev().for_each(|(sub, change)| {
            sub.iter().for_each(|sub| {
                scope.clear();
                scope.insert_state(StateId::ZERO);
                let Some(path) = self.tree.try_path(sub).cloned() else { return };

                update_tree(
                    &self.globals,
                    &self.factory,
                    &mut scope,
                    &mut self.states,
                    &mut self.components,
                    &change,
                    sub,
                    &path,
                    &mut self.tree,
                    &mut self.attribute_storage,
                    &mut self.floating_widgets,
                );
            });
        })
    }

    /// Perform a state changing operation.
    /// This will also apply future values
    pub fn with_state<F>(&mut self, state_id: impl Into<StateId>, f: F) -> &mut Self
    where
        F: FnOnce(&mut S),
    {
        let state_id = state_id.into();
        let state = self.states.get_mut(state_id).unwrap();
        f(state.to_any_mut().downcast_mut().unwrap());
        self.apply_futures();
        self.update_tree();

        let mut filter = LayoutFilter::new(false, &self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(self.text.new_session(), &self.attribute_storage, &self.viewport);
            layout_widget(
                widget,
                children,
                values,
                self.viewport.constraints(),
                &mut layout_ctx,
                false,
            );
        });

        // anathema_state::debug::Debug
        //     .heading()
        //     .header("owned")
        //     .print_owned()
        //     .header("shared")
        //     .print_shared()
        //     .header("tree")
        //     .print_tree::<anathema_widgets::DebugWidgets>(&mut self.tree)
        //     .footer();

        self
    }
}

pub struct TestCase {
    blueprint: Blueprint,
    globals: Globals,
}

impl TestCase {
    pub fn setup(src: &str) -> Self {
        let (blueprint, globals) = Document::new(src).compile().unwrap();
        Self { blueprint, globals }
    }

    pub fn build<S: 'static + State>(&self, state: S) -> TestCaseRunner<'_, S> {
        let tree = WidgetTree::empty();
        let components = ComponentRegistry::new();
        let mut states = States::new();
        states.insert(Box::new(state));

        let factory = setup_factory();

        let mut runner = TestCaseRunner {
            _p: PhantomData,
            globals: &self.globals,
            blueprint: &self.blueprint,
            tree,
            text: StringStorage::new(),
            states,
            components,
            factory,
            future_values: FutureValues::empty(),
            changes: Changes::empty(),
            attribute_storage: AttributeStorage::empty(),
            floating_widgets: FloatingWidgets::empty(),
            viewport: Viewport::new((1, 1)),
        };

        runner.exec();
        runner
    }
}

#[derive(Default)]
struct TestWidget;

impl Widget for TestWidget {
    fn layout<'bp>(
        &mut self,
        _children: LayoutChildren<'_, '_, 'bp>,
        _constraints: Constraints,
        _attributs: anathema_widgets::WidgetId,
        _ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        Size::new(1, 1)
    }

    fn position<'bp>(
        &mut self,
        _children: anathema_widgets::PositionChildren<'_, '_, 'bp>,
        _attributes: anathema_widgets::WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        _ctx: anathema_widgets::layout::PositionCtx,
    ) {
        todo!()
    }
}

pub(crate) fn setup_factory() -> Factory {
    let mut fac = Factory::new();
    fac.register_default::<TestWidget>("test");
    fac
}
