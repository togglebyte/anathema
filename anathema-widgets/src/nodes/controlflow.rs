use anathema_state::Change;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::{AttributeStorage, Value};

use crate::WidgetKind;
use crate::widget::WidgetTreeView;

#[derive(Debug)]
pub struct ControlFlow<'bp> {
    pub elses: Vec<Else<'bp>>,
}

impl<'bp> ControlFlow<'bp> {
    pub(crate) fn update(&mut self, change: &Change, branch_id: u16, attribute_storage: &AttributeStorage<'bp>) {
        match change {
            Change::Changed | Change::Dropped => {
                let Some(el) = self.elses.get_mut(branch_id as usize) else { return };
                let Some(cond) = el.cond.as_mut() else { return };
                cond.reload(attribute_storage)
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

impl ControlFlow<'_> {
    pub(crate) fn has_changed(&self, children: &WidgetTreeView<'_, '_>) -> bool {
        let child_count = children.layout_len();
        if child_count != 1 {
            return true;
        }

        let branch_id = self.current_branch_id(children);

        // Check if another branch id before this has become true,
        // if so this has changed.
        if self.elses[..branch_id as usize].iter().any(|e| e.is_true()) {
            return true;
        }

        // If the current branch is false, the value has changed,
        // as it has to have been true at one point to become
        // the current branch.
        !self.elses[branch_id as usize].is_true()
    }

    fn current_branch_id(&self, children: &WidgetTreeView<'_, '_>) -> u16 {
        let node_id = children.layout[0].value();
        let (_, widget) = children
            .values
            .get(node_id)
            .expect("because the node exists, the value exist");

        let WidgetKind::ControlFlowContainer(id) = widget.kind else { unreachable!() };
        id
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
