use anathema_render::Size;
use anathema_values::{Change, Collection, Context, NodeId, Path, Scope, ScopeValue, State};

use super::Nodes;
use crate::{WidgetContainer, contexts::LayoutCtx};

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct LoopNode {
    pub(super) body: Nodes,
    binding: Path,
    pub(super) collection: Collection,
    pub(super) value_index: usize,
}

impl LoopNode {
    pub(crate) fn new(body: Nodes, binding: Path, collection: Collection) -> Self {
        Self {
            body,
            binding,
            collection,
            value_index: 0,
        }
    }

    pub(super) fn reset_cache(&mut self) {
        self.body.reset_cache();
    }

    pub(super) fn count(&self) -> usize {
        self.body.count()
    }

    /// Scoping a value should only ever happen after an iteration
    pub(super) fn scope(&mut self, scope: &mut Scope) -> bool {
        if self.value_index >= self.collection.len() {
            return false;
        }
        scope.scope_collection(self.binding.clone(), &self.collection, self.value_index);
        self.body.expr_index = 0;
        self.value_index += 1;
        true
    }

    pub(super) fn remove(&mut self, index: usize) {
        self.collection.remove();
        self.value_index -= 1;
        if index >= self.body.inner.len() {
            return;
        }
        self.body.inner.remove(index);
    }

    pub(super) fn add(&mut self) {
        self.collection.add();
        // self.body.next_expr()
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes)> + '_ {
        self.body.iter_mut()
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        self.body.update(node_id, change, state)
    }
}
