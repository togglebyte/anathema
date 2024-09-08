use std::ops::ControlFlow;

use super::{Node, TreeValues, ValueId};

pub struct TreeForEach<'a, 'filter, T, Fil> {
    pub(super) nodes: &'a [Node],
    pub(super) values: &'a mut TreeValues<T>,
    pub(super) filter: &'filter Fil,
}

impl<'a, 'filter, T, Fil> TreeForEach<'a, 'filter, T, Fil> {
    pub fn new(nodes: &'a [Node], values: &'a mut TreeValues<T>, filter: &'filter Fil) -> Self {
        Self { nodes, values, filter }
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Fil::Output, TreeForEach<'_, '_, T, Fil>) -> ControlFlow<()>,
        Fil: TreeFilter<Input = T>,
    {
        self.inner_for_each(&mut f);
    }

    /// Apply to the first element that matches the filter
    pub fn first<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Fil::Output, &[Node], &mut TreeValues<T>),
        Fil: TreeFilter<Input = T>,
    {
        for node in self.nodes {
            self.values.with_mut(node.value(), |(_, value), values| {
                let filter = self.filter.filter(node.value(), value, node.children(), values);

                match filter {
                    ControlFlow::Break(()) => ControlFlow::Continue(()),
                    ControlFlow::Continue(None) => {
                        let mut for_each = TreeForEach {
                            nodes: node.children(),
                            values,
                            filter: self.filter,
                        };
                        for_each.first(f);
                        ControlFlow::Break(())
                    }
                    ControlFlow::Continue(Some(val)) => {
                        f(val, node.children(), values);
                        ControlFlow::Break(())
                    }
                }
            });
        }
    }

    fn inner_for_each<F>(&mut self, f: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&mut Fil::Output, TreeForEach<'_, '_, T, Fil>) -> ControlFlow<()>,
        Fil: TreeFilter<Input = T>,
    {
        for node in self.nodes {
            self.values.with_mut(node.value(), |(_, value), values| {
                let filter = self.filter.filter(node.value(), value, node.children(), values);

                match filter {
                    ControlFlow::Break(()) => ControlFlow::Continue(()),
                    ControlFlow::Continue(None) => {
                        let mut for_each = TreeForEach {
                            nodes: node.children(),
                            values,
                            filter: self.filter,
                        };
                        for_each.inner_for_each(f)
                    }
                    ControlFlow::Continue(Some(val)) => {
                        let each = TreeForEach {
                            nodes: node.children(),
                            values,
                            filter: self.filter,
                        };
                        f(val, each)
                    }
                }
            })?;
        }

        ControlFlow::Continue(())
    }
}

pub trait TreeFilter {
    type Input;
    type Output;

    fn filter<'val>(
        &self,
        value_id: ValueId,
        input: &'val mut Self::Input,
        children: &[Node],
        values: &mut TreeValues<Self::Input>,
    ) -> ControlFlow<(), Option<&'val mut Self::Output>>;
}
