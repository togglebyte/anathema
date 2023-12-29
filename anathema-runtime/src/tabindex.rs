use anathema_values::NodeId;
use anathema_widget_core::views::Views;

#[derive(Debug, Clone)]
struct TabIndex {
    node_id: NodeId,
    index: u32,
}

#[derive(Debug, Clone)]
struct TabIndexRef<'a> {
    node_id: &'a NodeId,
    index: u32,
}

impl From<&TabIndexRef<'_>> for TabIndex {
    fn from(value: &TabIndexRef<'_>) -> Self {
        TabIndex {
            index: value.index,
            node_id: value.node_id.to_owned(),
        }
    }
}

pub(super) enum Direction {
    Forwards,
    Backwards,
}

impl Direction {
    fn default(&self, max: usize) -> usize {
        match self {
            Self::Forwards => 0,
            Self::Backwards => max,
        }
    }

    fn next(&self, old: usize, max: usize) -> usize {
        match self {
            Self::Forwards if old == max => 0,
            Self::Backwards if old == 0 => max,
            Self::Forwards => old + 1,
            Self::Backwards => old - 1,
        }
    }
}

pub(super) struct TabIndexing {
    current_focus: Option<TabIndex>,
}

impl TabIndexing {
    pub(super) fn current_node(&self) -> Option<&NodeId> {
        self.current_focus.as_ref().map(|ti| &ti.node_id)
    }

    pub(super) fn new() -> Self {
        Self {
            current_focus: None,
        }
    }
}

impl TabIndexing {
    // Return the previously focused node so it can be "blurred".
    pub(super) fn next(&mut self, direction: Direction) -> Option<NodeId> {
        let views = Views::all();

        let mut views = views
            .iter()
            .filter_map(|f| {
                Some(TabIndexRef {
                    node_id: f.key(),
                    index: f.value?,
                })
            })
            .collect::<Vec<_>>();

        if views.is_empty() {
            return None;
        }

        views.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

        let default = direction.default(views.len() - 1);

        match self.current_focus.take() {
            None => {
                self.current_focus = Some(TabIndex::from(&views[default]));
                None
            }
            Some(old) => {
                let old_index = match views.binary_search_by(|idx| idx.index.cmp(&old.index)) {
                    Ok(i) => i,
                    Err(i) if i < views.len() => i,
                    Err(_) => default,
                };

                let next = direction.next(old_index, views.len() - 1);

                self.current_focus = Some(TabIndex::from(&views[next]));

                Some(old.node_id)
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn ids() -> Vec<(NodeId, u32)> {
        vec![
            (NodeId::from(10), 100u32),
            (NodeId::from(11), 101),
            (NodeId::from(2), 102),
        ]
    }

    fn insert_ids() {
        let node_ids = ids();

        for (id, index) in &node_ids {
            Views::test_insert(id.clone(), Some(*index));
        }
    }

    fn comp_index(index: &(NodeId, u32), tabs: &TabIndexing) {
        let current = tabs.current_node().unwrap();
        let current_index = tabs.current_focus.as_ref().unwrap().index;
        assert_eq!(current, &index.0);
        assert_eq!(current_index, index.1);
    }

    #[test]
    fn next_index() {
        let node_ids = ids();
        insert_ids();

        let mut tabs = TabIndexing::new();

        for index in &node_ids {
            tabs.next(Direction::Forwards);
            comp_index(index, &tabs);
        }
    }

    #[test]
    fn prev_index() {
        let mut node_ids = ids();
        insert_ids();

        let mut tabs = TabIndexing::new();
        node_ids.reverse();

        for index in &node_ids {
            tabs.next(Direction::Backwards);
            comp_index(index, &tabs);
        }
    }

    #[test]
    fn next_index_wrapping() {
        let node_ids = ids();
        insert_ids();

        let mut tabs = TabIndexing::new();

        node_ids
            .iter()
            .for_each(|_| drop(tabs.next(Direction::Forwards)));

        let last = tabs.next(Direction::Forwards).unwrap();
        assert_eq!(last, node_ids.last().unwrap().0);

        let current = tabs.current_node().unwrap();
        assert_eq!(current, &node_ids.first().unwrap().0);
    }

    #[test]
    fn insert_view() {
        let node_ids = ids();
        insert_ids();
        let mut tabs = TabIndexing::new();

        node_ids
            .iter()
            .for_each(|_| drop(tabs.next(Direction::Forwards)));

        let current = tabs.current_node().unwrap();
        assert_eq!(current, &node_ids.last().unwrap().0);

        Views::test_insert(NodeId::from(usize::MAX), Some(u32::MAX));
        let penultimate = tabs.next(Direction::Forwards).unwrap();
        assert_eq!(penultimate, node_ids.last().unwrap().0);

        let last = tabs.current_node().unwrap();
        assert_eq!(last, &NodeId::from(usize::MAX));
    }
}
