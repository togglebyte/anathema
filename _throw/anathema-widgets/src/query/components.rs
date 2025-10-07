use std::ops::ControlFlow;

use anathema_value_resolver::{AttributeStorage, Attributes};

use super::{Chain, Filter, Nodes, Query, QueryValue};
use crate::nodes::component::Component;
use crate::{WidgetId, WidgetKind};

pub struct Components<'children, 'tree, 'bp> {
    pub(super) elements: &'children mut Nodes<'tree, 'bp>,
}

impl<'children, 'tree, 'bp> Components<'children, 'tree, 'bp> {
    pub fn by_name<'a>(&mut self, name: &'a str) -> ComponentQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByName(name))
    }

    pub fn by_id<F, U>(&mut self, id: WidgetId, mut f: F) -> Option<U>
    where
        F: FnMut(&mut Component<'_>, &mut Attributes<'_>) -> U,
    {
        let widget = self.elements.children.get_mut(id)?;
        let WidgetKind::Component(component) = &mut widget.kind else { return None };
        let attributes = self.elements.attributes.get_mut(id);
        Some(f(component, attributes))
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
                elements: self.elements,
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
        match self.query(&mut f, false) {
            ControlFlow::Continue(_) => None,
            ControlFlow::Break(val) => Some(val),
        }
    }

    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(WidgetId, &mut Component<'_>, &mut Attributes<'_>),
    {
        _ = self.query(&mut f, true);
    }

    fn query<F, U>(self, f: &mut F, continuous: bool) -> ControlFlow<U>
    where
        F: FnMut(WidgetId, &mut Component<'_>, &mut Attributes<'_>) -> U,
    {
        let ret_val = self.query.elements.children.for_each(|_path, container, children| {
            if let WidgetKind::Component(ref mut component) = container.kind {
                if self.query.filter.filter(component, self.query.elements.attributes) {
                    let attributes = self.query.elements.attributes.get_mut(component.widget_id);
                    let ret_val = f(component.widget_id, component, attributes);

                    if !continuous {
                        return ControlFlow::Break(ret_val);
                    }
                }
            }

            let mut elements = Nodes::new(
                children,
                self.query.elements.attributes,
                self.query.elements.dirty_widgets,
            );

            let query = ComponentQuery {
                query: Query {
                    elements: &mut elements,
                    filter: self.query.filter,
                },
            };

            query.query(f, continuous)
        });

        match ret_val {
            Some(val) => ControlFlow::Break(val),
            None => ControlFlow::Continue(()),
        }
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
                attribs.get(key).map(|attribute| value.eq(attribute)).unwrap_or(false)
            }
        }
    }
}
