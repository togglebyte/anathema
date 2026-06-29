use anathema_store::remotecell::RemoteCell;

use crate::eval::blueprints::values::TemplateValue;

#[derive(Debug)]
pub struct ControlFlow<'bp> {
    cond: RemoteCell<TemplateValue<'bp>>,
}
