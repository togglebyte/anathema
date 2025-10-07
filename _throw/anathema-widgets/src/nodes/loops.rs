use anathema_state::Change;
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::Collection;

use super::{WidgetContainer, WidgetKind};
use crate::error::{Error, Result};
use crate::layout::LayoutCtx;
use crate::widget::WidgetTreeView;
use crate::{DirtyWidgets, WidgetId};

#[derive(Debug)]
pub struct For<'bp> {
    pub(crate) binding: &'bp str,
    pub(crate) collection: Collection<'bp>,
    pub(crate) body: &'bp [Blueprint],
}

impl<'bp> For<'bp> {
    pub fn binding(&self) -> &'bp str {
        self.binding
    }

    pub(super) fn update(
        &mut self,
        change: &Change,
        mut tree: WidgetTreeView<'_, 'bp>,
        ctx: &mut LayoutCtx<'_, 'bp>,
        parent_widget: Option<WidgetId>,
        dirty_widgets: &mut DirtyWidgets,
    ) -> Result<()> {
        if let Some(widget) = parent_widget {
            dirty_widgets.push(widget);
        }

        match change {
            Change::Inserted(index) => {
                // 1. Declare insert path
                // 2. Create new iteration
                // 3. Insert new iteration
                // 4. Update index of all subsequent iterations

                // ctx.scope.push();
                // ctx.scope.scope_pending(self.binding, *value);

                let path = [*index as u16];
                let transaction = tree.insert(&path);
                let widget = WidgetKind::Iteration(Iteration {
                    loop_index: anathema_state::Value::new(*index as i64),
                    binding: self.binding,
                });
                let widget = WidgetContainer::new(widget, self.body, parent_widget);
                let _ = transaction.commit_at(widget).ok_or_else(Error::transaction_failed)?;

                for child in &tree.layout[*index as usize + 1..] {
                    let iter_widget = tree.values.get_mut(child.value());
                    let Some((
                        _,
                        WidgetContainer {
                            kind: WidgetKind::Iteration(iter),
                            ..
                        },
                    )) = iter_widget
                    else {
                        unreachable!("this can only ever be an iteration")
                    };
                    *iter.loop_index.to_mut() += 1;
                }
            }
            Change::Removed(index) => {
                if let Some(widget) = parent_widget {
                    dirty_widgets.push(widget);
                }

                for child in &tree.layout[*index as usize + 1..] {
                    let iter_widget = tree.values.get_mut(child.value());
                    let Some((
                        _,
                        WidgetContainer {
                            kind: WidgetKind::Iteration(iter),
                            ..
                        },
                    )) = iter_widget
                    else {
                        unreachable!("this can only ever be an iteration")
                    };
                    *iter.loop_index.to_mut() -= 1;
                }

                ctx.relative_remove(&[*index as u16], tree);
            }
            Change::Dropped | Change::Changed => {
                // If the collection has changed to a different collection
                // then truncate the tree
                self.collection.reload(ctx.attribute_storage);
                ctx.truncate_children(&mut tree);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Iteration<'bp> {
    pub loop_index: anathema_state::Value<i64>,
    pub binding: &'bp str,
}
