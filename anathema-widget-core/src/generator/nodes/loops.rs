use anathema_render::Size;
use anathema_values::{Change, Collection, Context, NodeId, Path, Scope, ScopeValue, State, ValueExpr};

use super::Nodes;
use crate::{WidgetContainer, contexts::LayoutCtx};

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct LoopNode<'e> {
    pub(super) body: Nodes<'e>,
    binding: Path,
    pub(super) collection: &'e ValueExpr,
    pub(super) value_index: usize,
}

impl<'e> LoopNode<'e> {
    pub(crate) fn new(body: Nodes<'e>, binding: Path, collection: &'e ValueExpr) -> Self {
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
        panic!()
        // if self.value_index >= self.collection.len() {
        //     return false;
        // }
        // scope.scope_collection(self.binding.clone(), &self.collection, self.value_index);
        // self.body.expr_index = 0;
        // self.value_index += 1;
        // true
    }

    pub(super) fn remove(&mut self, index: usize) {
        panic!()
        // self.collection.remove();
        // if index >= self.body.inner.len() {
        //     return;
        // }
        // self.value_index -= 1;
        // self.body.inner.remove(index);
    }

    pub(super) fn add(&mut self) {
        panic!()
        // self.collection.add();
        // self.body.next_expr()
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer, &mut Nodes<'e>)> + '_ {
        self.body.iter_mut()
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        self.body.update(node_id, change, state)
    }
}
