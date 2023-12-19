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
    // Return the previously focused node so it can be "blurred"
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
