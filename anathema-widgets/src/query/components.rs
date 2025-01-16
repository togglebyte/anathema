use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region};
use anathema_state::CommonVal;
pub use anathema_store::tree::visitor::apply_visitor;
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::{Node, TreeValues};
use anathema_value_resolver::{AttributeStorage, Attributes};

use super::{Chain, Filter, Nodes, Query, QueryValue};
use crate::nodes::component::Component;
use crate::nodes::element::Element;
use crate::{DirtyWidgets, WidgetContainer, WidgetId, WidgetKind, WidgetTreeView};

fn comptest_delete_this<'tree, 'bp>(components: ComponentQuery<'_, 'tree, 'bp, Kind<'bp>>) {
    let value = components.by_name("").by_name("").by_name("").first(|id, c, a| 123);
}

pub struct Components<'tree, 'bp> {
    elements: Nodes<'tree, 'bp>,
}

impl<'tree, 'bp> Components<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        dirty_widgets: &'tree mut DirtyWidgets,
    ) -> Self {
        Self {
            elements: Nodes::new(children, attribute_storage, dirty_widgets),
        }
    }

    pub fn by_name<'a>(&mut self, name: &'a str) -> ComponentQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByName(name))
    }

    pub fn by_attribute<'a>(
        &mut self,
        key: &'a str,
        value: impl Into<QueryValue<'a>>,
    ) -> ComponentQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByAttribute(key, value.into()))
    }

    fn make_query<'a>(&mut self, kind: Kind<'a>) -> ComponentQuery<'_, 'tree, 'bp, Kind<'a>> {
        ComponentQuery {
            query: Query {
                filter: kind,
                elements: &mut self.elements,
            },
        }
    }
}

pub struct ComponentQuery<'el, 'tree, 'bp, T>
where
    T: Filter<'bp, Kind = Component<'bp>> + Copy,
{
    query: Query<'el, 'tree, 'bp, T, Component<'bp>>,
}

impl<'el, 'tree, 'bp, T> ComponentQuery<'el, 'tree, 'bp, T>
where
    T: Filter<'bp, Kind = Component<'bp>> + Copy,
{
    pub fn by_name(self, name: &str) -> ComponentQuery<'el, 'tree, 'bp, Chain<T, Kind<'_>>> {
        ComponentQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::ByName(name)),
                elements: self.query.elements,
            },
        }
    }

    pub fn by_attribute<'a>(
        self,
        key: &'a str,
        value: impl Into<QueryValue<'a>>,
    ) -> ComponentQuery<'el, 'tree, 'bp, Chain<T, Kind<'a>>> {
        ComponentQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::ByAttribute(key, value.into())),
                elements: self.query.elements,
            },
        }
    }

    pub fn first<F, U>(self, mut f: F) -> Option<U>
    where
        F: FnMut(WidgetId, &mut Component<'_>, &mut Attributes<'_>) -> U,
    {
        self.query(&mut f, false)
    }

    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(WidgetId, &mut Component<'_>, &mut Attributes<'_>),
    {
        self.query::<()>(&mut f, true);
    }

    fn query<U>(
        self,
        f: &mut impl FnMut(WidgetId, &mut Component<'_>, &mut Attributes<'_>) -> U,
        continuous: bool,
    ) -> Option<U> {
        for i in 0..self.query.elements.children.layout.len() {
            let node = &self.query.elements.children.layout[i];
            let Some((_path, container)) = self.query.elements.children.values.get_mut(node.value()) else {
                continue;
            };

            let WidgetKind::Component(ref mut component) = container.kind else { continue };

            if !self.query.filter.filter(component, self.query.elements.attributes) {
                continue;
            }

            let attributes = self.query.elements.attributes.get_mut(component.widget_id);
            let retval = f(component.widget_id, component, attributes);

            if !continuous {
                return Some(retval);
            }

            let mut query = ComponentQuery {
                query: Query {
                    elements: self.query.elements,
                    filter: self.query.filter,
                },
            };

            query.query(f, continuous);
        }

        None
    }
}

// -----------------------------------------------------------------------------
//   - Query kind -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum Kind<'a> {
    ByName(&'a str),
    ByAttribute(&'a str, QueryValue<'a>),
}

impl<'bp> Filter<'bp> for Kind<'_> {
    type Kind = Component<'bp>;

    fn filter(&self, el: &Self::Kind, attributes: &mut AttributeStorage<'bp>) -> bool {
        match self {
            Kind::ByName(name) => el.name == *name,
            Kind::ByAttribute(key, value) => {
                let attribs = attributes.get(el.widget_id);
                attribs
                    .get(key)
                    .map(|attribute| value.eq(attribute))
                    .unwrap_or(false)
            }
        }
    }
}
