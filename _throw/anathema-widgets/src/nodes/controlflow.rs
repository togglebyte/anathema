use anathema_state::Change;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::Value;

use crate::layout::LayoutCtx;
use crate::widget::WidgetTreeView;
use crate::{DirtyWidgets, WidgetId};

#[derive(Debug)]
pub struct ControlFlow<'bp> {
    pub elses: Vec<Else<'bp>>,
}

impl<'bp> ControlFlow<'bp> {
    pub(crate) fn update(
        &mut self,
        change: &Change,
        branch_id: u16,
        mut tree: WidgetTreeView<'_, 'bp>,
        ctx: &mut LayoutCtx<'_, 'bp>,
        parent_widget: Option<WidgetId>,
        dirty_widgets: &mut DirtyWidgets,
    ) {
        match change {
            Change::Changed | Change::Dropped => {
                let Some(el) = self.elses.get_mut(branch_id as usize) else { return };
                let Some(cond) = el.cond.as_mut() else { return };
                let current = cond.truthiness();
                cond.reload(ctx.attribute_storage);
                if cond.truthiness() != current {
                    ctx.truncate_children(&mut tree);

                    if let Some(widget) = parent_widget {
                        dirty_widgets.push(widget);
                    }
                }
            }
            // TODO:
            // this could probably happen given something like this
            // ```
            // if state.list
            //     text "list is not empty"
            // ```
            Change::Inserted(_) | Change::Removed(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Else<'bp> {
    pub cond: Option<Value<'bp>>,
    pub body: &'bp [Blueprint],
    pub show: bool,
}

impl Else<'_> {
    pub(crate) fn is_true(&self) -> bool {
        match self.cond.as_ref() {
            Some(cond) => cond.truthiness(),
            None => true,
        }
    }
}
