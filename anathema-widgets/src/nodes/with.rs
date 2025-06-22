use anathema_state::Change;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::{AttributeStorage, Collection, Value};

use super::{WidgetContainer, WidgetKind};
use crate::error::{Error, Result};
use crate::widget::WidgetTreeView;

#[derive(Debug)]
pub struct With<'bp> {
    pub(crate) binding: &'bp str,
    pub(crate) data: Value<'bp>,
    pub(crate) body: &'bp [Blueprint],
}

impl<'bp> With<'bp> {
    pub fn binding(&self) -> &'bp str {
        self.binding
    }

    pub(super) fn update(
        &mut self,
        change: &Change,
        mut tree: WidgetTreeView<'_, 'bp>,
        attribute_storage: &mut AttributeStorage<'bp>,
    ) -> Result<()> {
        match change {
            Change::Dropped | Change::Changed => {
                self.data.reload(attribute_storage);
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Iteration<'bp> {
    pub loop_index: anathema_state::Value<i64>,
    pub binding: &'bp str,
}

