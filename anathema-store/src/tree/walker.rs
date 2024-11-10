use std::marker::PhantomData;

use super::view::TreeView;

struct Walker<'tree, C, T, Fil> {
    _p: PhantomData<C>,
    filter: Fil,
    treeview: TreeView<'tree, T>,
}

impl<'tree, C, T, Fil> Walker<'tree, C, T, Fil> {
    fn for_each<F>(&mut self, ctx: &mut C, mut f: F)
    where
        F: FnMut(&mut T, Self),
    {
        // let Some(value, children) = self.filter.filter_or_generate(ctx) else { panic!("what do we do here?") };
        // f(value, children);
    }
}
