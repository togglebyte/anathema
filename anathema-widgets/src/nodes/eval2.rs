use anathema_store::tree::TreeView;
use anathema_templates::blueprints::Blueprint;

use super::WidgetContainer;
use crate::error::{Error, Result};
use crate::nodes::eval::{ComponentEval, ControlFlowEval, Evaluator, ForLoopEval, SingleEval};
use crate::EvalContext;

pub type WidgetTree<'tree, 'bp> = TreeView<'tree, WidgetContainer<'bp>>;

pub fn eval_blueprint<'bp>(
    blueprint: &'bp Blueprint,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    parent: &[u16],
    tree: &mut WidgetTree<'_, 'bp>,
) -> Result<()> {
    match blueprint {
        _ => panic!("test knows best")
        // Blueprint::Single(single) => SingleEval.eval(single, ctx, parent, tree),
        // Blueprint::For(for_loop) => ForLoopEval.eval(for_loop, ctx, parent, tree),
        // Blueprint::ControlFlow(flow) => ControlFlowEval.eval(flow, ctx, parent, tree),
        // Blueprint::Component(component) => ComponentEval.eval(component, ctx, parent, tree),
    }

    panic!()
}
