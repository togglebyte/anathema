use crate::runtime::elements::ElementId;
use crate::runtime::eval::EvalCtx;
use crate::templates::Expression;
use crate::testing::with_blueprint;

pub fn with_expr<F>(expr: Expression, f: F)
where
    for <'bp> F: Fn(&'bp Expression, ElementId, &mut EvalCtx<'_, 'bp>),
{
    // with_blueprint(|

}
