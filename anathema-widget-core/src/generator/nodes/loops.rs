use anathema_values::{Change, Context, Path, State, ValueExpr, ValueRef};

use super::Nodes;
use crate::generator::expressions::Collection;
use crate::WidgetContainer;

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct LoopNode<'e> {
    pub(super) body: Nodes<'e>,
    pub(super) binding: Path,
    pub(super) collection: Collection<'e>,
    pub(super) value_index: usize,
}

impl<'e> LoopNode<'e> {
    pub(crate) fn new(body: Nodes<'e>, binding: Path, collection: Collection<'e>) -> Self {
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

    pub(super) fn next_value(&mut self, context: &Context<'_, 'e>) -> Option<ValueRef<'e>> {
        let val = match self.collection {
            Collection::ValueExpressions(expressions) => expressions.get(self.value_index)?.eval_value_ref(context)?,
            Collection::Path(ref path) => context.lookup(path)?,
            Collection::State { len, .. } if len == self.value_index => return None,
            Collection::State { len, ref path } => ValueRef::Deferred(path.compose(self.value_index)),
            // TODO: remove comments. 2023-10-27
            // ValueRef::Expressions(list) => list.get(self.value_index)?.eval_value(context, None)?,
            // ValueRef::List(list) => list.get(&Path::Index(self.value_index), None)?,
            Collection::Empty => return None,
        };
        self.value_index += 1;
        Some(val)
    }

    pub(super) fn remove(&mut self, _index: usize) {
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
    ) -> impl Iterator<Item = (&mut WidgetContainer<'e>, &mut Nodes<'e>)> + '_ {
        self.body.iter_mut()
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, state: &mut impl State) {
        self.body.update(node_id, change, state)
    }
}
