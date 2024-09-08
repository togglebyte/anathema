use std::marker::PhantomData;
use std::ops::ControlFlow;

use anathema_geometry::{Pos, Size};
use anathema_state::{drain_changes, drain_futures, Changes, FutureValues, State, StateId, States};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Components, DirtyWidgets, Elements,
    EvalContext, Factory, FloatingWidgets, LayoutChildren, Scope, Stringify, Widget, WidgetTree,
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
    dirty_widgets: DirtyWidgets,
    floating_widgets: FloatingWidgets,
    states: States,
    component_registry: ComponentRegistry,
    future_values: FutureValues,
    changes: Changes,
    viewport: Viewport,
    components: Components,
}

impl<'bp, S> TestCaseRunner<'bp, S>
where
    S: 'static + State,
{
    fn exec(&mut self) {
        let mut scope = Scope::new();
        scope.insert_state(StateId::ZERO);
        let mut ctx = EvalContext::new(
            self.globals,
            &self.factory,
            &mut scope,
            &mut self.states,
            &mut self.component_registry,
            &mut self.attribute_storage,
            &mut self.floating_widgets,
            &mut self.components,
        );

        eval_blueprint(self.blueprint, &mut ctx, &[], &mut self.tree).unwrap();

        // Non floating widgets
        let mut filter = LayoutFilter::new(true, &self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(&self.attribute_storage, &self.viewport);
            layout_widget(
                widget,
                children,
                values,
                self.viewport.constraints(),
                &mut layout_ctx,
                true,
            );

            position_widget(
                Pos::ZERO,
                widget,
                children,
                values,
                &self.attribute_storage,
                true,
                self.viewport,
            );
        });

        // Floating widgets
        let mut filter = LayoutFilter::new(false, &self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            let mut layout_ctx = LayoutCtx::new(&self.attribute_storage, &self.viewport);
            layout_widget(
                widget,
                children,
                values,
                self.viewport.constraints(),
                &mut layout_ctx,
                true,
            );

            position_widget(
                Pos::ZERO,
                widget,
                children,
                values,
                &self.attribute_storage,
                true,
                self.viewport,
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
            let path = self.tree.path(sub);

            try_resolve_future_values(
                self.globals,
                &self.factory,
                &mut scope,
                &mut self.states,
                &mut self.component_registry,
                sub,
                &path,
                &mut self.tree,
                &mut self.attribute_storage,
                &mut self.floating_widgets,
                &mut self.components,
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
                let Some(path) = self.tree.try_path(sub) else { return };

                update_tree(
                    self.globals,
                    &self.factory,
                    &mut scope,
                    &mut self.states,
                    &mut self.component_registry,
                    &change,
                    sub,
                    &path,
                    &mut self.tree,
                    &mut self.attribute_storage,
                    &mut self.floating_widgets,
                    &mut self.components,
                );
            });
        })
    }

    /// Perform a state changing operation.
    /// This will also apply future values
    #[allow(dead_code)]
    pub fn with_query<F>(&mut self, state_id: impl Into<StateId>, f: F) -> &mut Self
    where
        for<'a, 'b> F: FnOnce(&mut S, Elements<'a, 'b>),
    {
        let state_id = state_id.into();
        let state = self.states.get_mut(state_id).unwrap();

        let Some((node, values)) = self.tree.get_node_by_path(&[0]) else { return self };
        let elements = Elements::new(
            node.children(),
            values,
            &mut self.attribute_storage,
            &mut self.dirty_widgets,
        );

        let state = state.to_any_mut().downcast_mut().unwrap();
        f(state, elements);

        self.apply_futures();
        self.update_tree();

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

    /// Perform a state changing operation.
    /// This will also apply future values
    #[allow(dead_code)]
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
            let mut layout_ctx = LayoutCtx::new(&self.attribute_storage, &self.viewport);
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
            states,
            component_registry: components,
            factory,
            future_values: FutureValues::empty(),
            changes: Changes::empty(),
            attribute_storage: AttributeStorage::empty(),
            floating_widgets: FloatingWidgets::empty(),
            viewport: Viewport::new((1, 1)),
            components: Components::new(),
            dirty_widgets: DirtyWidgets::empty(),
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
        mut children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        _attributs: anathema_widgets::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let mut size = Size::new(1, 1);

        children.for_each(|node, children| {
            let widget_size = node.layout(children, constraints, ctx);
            size.width = size.width.max(widget_size.width);
            size.height += widget_size.height;

            ControlFlow::Continue(())
        });

        size
    }

    fn position<'bp>(
        &mut self,
        mut children: anathema_widgets::PositionChildren<'_, '_, 'bp>,
        _attributes: anathema_widgets::WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: anathema_widgets::layout::PositionCtx,
    ) {
        let mut pos = Pos::ZERO;
        children.for_each(|node, children| {
            node.position(children, pos, attribute_storage, ctx.viewport);
            pos.y += node.size().height as i32;

            ControlFlow::Continue(())
        });
    }
}

pub(crate) fn setup_factory() -> Factory {
    let mut fac = Factory::new();
    fac.register_default::<TestWidget>("test");
    fac
}
