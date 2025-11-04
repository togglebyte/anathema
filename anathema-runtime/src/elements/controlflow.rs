use anathema_store::remotecell::RemoteCell;

use crate::eval::values::TemplateValue;

#[derive(Debug)]
pub struct ControlFlow<'bp> {
    cond: RemoteCell<TemplateValue<'bp>>,
}
