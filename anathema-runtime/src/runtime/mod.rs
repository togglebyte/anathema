use anathema_compiler::expressions::Expressions;
use anathema_compiler::{Document, Variables};
use anathema_frontend::Frontend;
use anathema_geometry::Pos;

use crate::attributes::AttributeRegistry;
use crate::components::Components;
use crate::elements::Elements;
use crate::eval::blueprints::expression::RuntimeExpressions;
use crate::eval::blueprints::scope::Scope;
use crate::eval::blueprints::{BlueprintEvalCtx, eval_blueprint};
use crate::widgets::{Layouts, RegisteredWidgets, Root};
use crate::{Constraints, FunctionTable};

pub struct Runtime<Fe> {
    frontend: Fe,
    doc: Document,
    widget_reg: RegisteredWidgets,
}

impl<Fe> Runtime<Fe> {
    pub fn new(doc: Document, frontend: Fe, widget_reg: RegisteredWidgets) -> Self {
        Self {
            frontend,
            doc,
            widget_reg,
        }
    }

    pub fn run(mut self, mut components: Components)
    where
        Fe: Frontend,
    {
        let mut globals = Variables::new();
        let functions = FunctionTable::new();
        let bp = self.doc.compile(&mut globals).unwrap();
        let mut elements = Elements::empty();
        let mut attributes = AttributeRegistry::empty();
        let expressions = &self.doc.expressions;
        let mut runtime_expressions = RuntimeExpressions::empty();
        let mut scope = Scope::empty();
        let mut dirty_elements = vec![];

        let root_id = elements.insert_root();

        let mut ctx = BlueprintEvalCtx::new(
            &mut elements,
            &mut attributes,
            &mut components,
            &globals,
            expressions,
            &functions,
            &mut scope,
            &mut runtime_expressions,
            &mut dirty_elements,
        );

        eval_blueprint(&bp, &mut ctx, &self.widget_reg, Some(root_id)).unwrap();

        let root = elements.root();
        let root_attributes = crate::Attributes::empty();
        let root_attributes = crate::WidgetAttributes::new(&elements, None, &root_attributes, &attributes);
        let crate::elements::Element::Widget(root_widget) = &root.element else { unreachable!() };

        loop {
            let constraints = Constraints::new(self.frontend.viewport_size());
            let children = crate::widgets::Children::new(&root.children, &elements, &attributes);
            let mut layouts = Layouts::empty();
            let mut root_widget_ref = crate::widgets::iter::WidgetRef::new(
                elements.root,
                root_widget.borrow_mut(),
                root_attributes,
                children,
            );

            root_widget_ref.layout(&mut layouts, constraints);
            root_widget_ref.position(&mut layouts, Pos::ZERO);
            root_widget_ref.paint(&mut self.frontend, &layouts);

            // * [x] Layout
            // * [x] Position
            // * [x] Paint
            // * [ ] Events
            // * [ ] Messages
            // * [ ] Deferred events

            self.frontend.render();
            std::thread::sleep_ms(1020);
        }
    }
}
