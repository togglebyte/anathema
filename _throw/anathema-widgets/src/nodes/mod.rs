use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::Scope;
use eval::SlotEval;

pub use self::element::Element;
use self::eval::{ComponentEval, ControlFlowEval, Evaluator, ForLoopEval, SingleEval};
pub use self::update::update_widget;
use crate::WidgetId;
use crate::error::Result;
use crate::layout::EvalCtx;
use crate::nodes::eval::WithEval;
use crate::widget::WidgetTreeView;

pub(crate) mod component;
pub(crate) mod controlflow;
pub(crate) mod element;
pub(crate) mod eval;
pub(crate) mod loops;
mod update;
pub(crate) mod with;

// -----------------------------------------------------------------------------
//   - Generators -
// -----------------------------------------------------------------------------
pub enum WidgetGenerator<'bp> {
    Children(&'bp [Blueprint]),
    Single,
    Slot(&'bp [Blueprint]),
    Loop(&'bp [Blueprint]),
    ControlFlow,
    Noop,
}

#[derive(Debug)]
pub enum WidgetKind<'bp> {
    Element(Element<'bp>),
    For(loops::For<'bp>),
    With(with::With<'bp>),
    Iteration(loops::Iteration<'bp>),
    ControlFlow(controlflow::ControlFlow<'bp>),
    ControlFlowContainer(u16),
    Component(component::Component<'bp>),
    Slot,
}

#[derive(Debug)]
pub struct WidgetContainer<'bp> {
    pub kind: WidgetKind<'bp>,
    pub(crate) children: &'bp [Blueprint],
    pub parent_widget: Option<WidgetId>,
}

impl<'bp> WidgetContainer<'bp> {
    pub fn new(kind: WidgetKind<'bp>, blueprints: &'bp [Blueprint], parent_widget: Option<WidgetId>) -> Self {
        Self {
            kind,
            children: blueprints,
            parent_widget,
        }
    }
}

pub fn eval_blueprint<'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalCtx<'_, 'bp>,
    scope: &Scope<'_, 'bp>,
    parent: &[u16],
    tree: &mut WidgetTreeView<'_, 'bp>,
) -> Result<()> {
    match blueprint {
        Blueprint::Single(single) => SingleEval.eval(single, ctx, scope, parent, tree),
        Blueprint::For(for_loop) => ForLoopEval.eval(for_loop, ctx, scope, parent, tree),
        Blueprint::With(with) => WithEval.eval(with, ctx, scope, parent, tree),
        Blueprint::ControlFlow(flow) => ControlFlowEval.eval(flow, ctx, scope, parent, tree),
        Blueprint::Component(component) => ComponentEval.eval(component, ctx, scope, parent, tree),
        Blueprint::Slot(blueprints) => SlotEval.eval(blueprints, ctx, scope, parent, tree),
    }
}
