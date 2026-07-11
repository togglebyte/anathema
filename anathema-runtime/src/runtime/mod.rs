use std::time::{Duration, Instant};

use anathema_compiler::blueprints::Blueprint;
use anathema_compiler::expressions::Expressions;
use anathema_compiler::{Document, Variables};
use anathema_frontend::Frontend;
use anathema_geometry::Pos;

use crate::attributes::AttributeRegistry;
use crate::components::Components;
use crate::elements::{Element, Elements};
use crate::eval::blueprints::expression::RuntimeExpressions;
use crate::eval::blueprints::scope::Scope;
use crate::eval::blueprints::{BlueprintEvalCtx, eval_blueprint};
use crate::widgets::iter::WidgetRef;
use crate::widgets::{Layouts, RegisteredWidgets, Root, View, build_view_tree};
use crate::{Constraints, ElementId, FunctionTable};

pub struct Runtime<Fe> {
    frontend: Fe,
    doc: Document,
    blueprint: Blueprint,
    widget_reg: RegisteredWidgets,
    globals: Variables,
    functions: FunctionTable,
    components: Components,
    dirty_elements: Vec<ElementId>,
}

impl<Fe> Runtime<Fe> {
    pub fn new(
        doc: Document,
        blueprint: Blueprint,
        globals: Variables,
        components: Components,
        frontend: Fe,
        widget_reg: RegisteredWidgets,
    ) -> Self {
        Self {
            frontend,
            doc,
            blueprint,
            widget_reg,
            globals,
            functions: FunctionTable::new(),
            components,
            dirty_elements: vec![],
        }
    }

    pub fn instance(&mut self) -> RuntimeInstance<'_, '_, Fe> {
        RuntimeInstance::new(
            &mut self.components,
            &self.doc.expressions,
            &self.functions,
            &mut self.dirty_elements,
            &self.globals,
            &mut self.frontend,
            &self.blueprint,
            &self.widget_reg,
        )
    }

    pub fn run(&mut self)
    where
        Fe: Frontend,
    {
        loop {
            let mut instance = self.instance();
            instance.run();
        }
    }
}

// -----------------------------------------------------------------------------
//   - Instance -
// -----------------------------------------------------------------------------
pub struct RuntimeInstance<'rt, 'bp, Fe> {
    elements: Elements<'bp>,
    attributes: AttributeRegistry<'bp>,
    runtime_expressions: RuntimeExpressions<'bp>,
    scope: Scope<'bp>,
    components: &'rt mut Components,
    expressions: &'bp Expressions,
    functions: &'bp FunctionTable,
    dirty_elements: &'rt mut Vec<ElementId>,
    variables: &'rt Variables,
    frontend: &'rt mut Fe,
    blueprint: &'bp Blueprint,
    widget_reg: &'rt RegisteredWidgets,
}

impl<'rt, 'bp, Fe> RuntimeInstance<'rt, 'bp, Fe> {
    pub fn new(
        components: &'rt mut Components,
        expressions: &'bp Expressions,
        functions: &'bp FunctionTable,
        dirty_elements: &'rt mut Vec<ElementId>,
        variables: &'rt Variables,
        frontend: &'rt mut Fe,
        blueprint: &'bp Blueprint,
        widget_reg: &'rt RegisteredWidgets,
    ) -> Self {
        let mut attributes = AttributeRegistry::empty();

        let mut elements = Elements::with_root();
        let root = elements.root();
        let root_attributes = crate::Attributes::empty();
        let root_attributes = crate::WidgetAttributes::new(&elements, None, &root_attributes, &attributes);
        let Element::Widget(root_widget) = &root.element else { unreachable!() };
        let view = View::root(elements.root);

        Self {
            elements,
            attributes,
            runtime_expressions: RuntimeExpressions::empty(),
            scope: Scope::empty(),
            components,
            expressions,
            functions,
            dirty_elements,
            variables,
            frontend,
            blueprint,
            widget_reg
        }
    }

    fn context(&mut self) -> BlueprintEvalCtx<'_, 'bp> {
        BlueprintEvalCtx::new(
            &mut self.elements,
            &mut self.attributes,
            self.components,
            self.variables,
            self.expressions,
            &self.functions,
            &mut self.scope,
            &mut self.runtime_expressions,
            self.dirty_elements,
        )
    }

    pub fn tick(&mut self) -> Duration
    where
        Fe: Frontend,
    {
        let now = Instant::now();
        let constraints = Constraints::new(self.frontend.viewport_size());
        let children = crate::widgets::Children::new(&self.elements.root().children, &self.elements, &self.attributes);
        let mut layouts = Layouts::empty();

        let Element::Widget(root_widget) = &self.elements.root().element else { unreachable!() };
        let root_attributes = crate::Attributes::empty();
        let root_attributes = crate::WidgetAttributes::new(&self.elements, None, &root_attributes, &self.attributes);

        let mut root_widget_ref =
            WidgetRef::new(self.elements.root, root_widget.borrow_mut(), root_attributes, children);

        root_widget_ref.layout(&mut layouts, constraints);
        root_widget_ref.position(&mut layouts, Pos::ZERO);
        root_widget_ref.paint(self.frontend, &layouts);

        // * [x] Layout
        // * [x] Position
        // * [x] Paint
        // * [ ] Events
        // * [ ] Messages
        // * [ ] Deferred events

        now.elapsed()
    }

    fn run(mut self)
    where
        Fe: Frontend,
    {
        // Evaluate blueprints
        let root = self.elements.root;
        let blueprint = self.blueprint;
        let widget_reg = self.widget_reg;
        let mut ctx = self.context();
        eval_blueprint(blueprint, &mut ctx, widget_reg, root).unwrap();

        // Build the view tree
        let view = build_view_tree(root, &self.elements);

        loop {
            self.tick();
            self.frontend.render();
            std::thread::sleep_ms(50);
        }
    }
}
