use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region};
use anathema_value_resolver::{AttributeStorage, Attributes};

use super::{Chain, Filter, Nodes, Query, QueryValue};
use crate::nodes::element::Element;
use crate::{WidgetId, WidgetKind};

pub struct Elements<'children, 'tree, 'bp> {
    pub(super) elements: &'children mut Nodes<'tree, 'bp>,
}

impl<'children, 'tree, 'bp> Elements<'children, 'tree, 'bp> {
    pub fn by_tag<'a>(&mut self, tag: &'a str) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByTag(tag))
    }

    pub fn at_position<'a>(&mut self, pos: impl Into<Pos>) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::AtPosition(pos.into()))
    }

    pub fn first<F, U>(&mut self, f: F) -> Option<U>
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>) -> U,
    {
        self.make_query(Kind::All).first(f)
    }

    pub fn each<F>(&mut self, f: F)
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>),
    {
        self.make_query(Kind::All).each(f)
    }

    pub fn by_attribute<'a>(
        &mut self,
        key: &'a str,
        value: impl Into<QueryValue<'a>>,
    ) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByAttribute(key, value.into()))
    }

    pub fn by_id<'a>(&mut self, id: WidgetId) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ById(id))
    }

    fn make_query<'a>(&mut self, kind: Kind<'a>) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        ElementQuery {
            query: Query {
                filter: kind,
                elements: self.elements,
            },
        }
    }
}

pub struct ElementQuery<'el, 'tree, 'bp, T>
where
    T: Filter<'bp, Kind = Element<'bp>> + Copy,
{
    query: Query<'el, 'tree, 'bp, T, Element<'bp>>,
}

impl<'el, 'tree, 'bp, T> ElementQuery<'el, 'tree, 'bp, T>
where
    T: Filter<'bp, Kind = Element<'bp>> + Copy,
{
    pub fn by_tag(self, name: &str) -> ElementQuery<'el, 'tree, 'bp, Chain<T, Kind<'_>>> {
        ElementQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::ByTag(name)),
                elements: self.query.elements,
            },
        }
    }

    pub fn at_position(self, pos: impl Into<Pos>) -> ElementQuery<'el, 'tree, 'bp, Chain<T, Kind<'static>>> {
        ElementQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::AtPosition(pos.into())),
                elements: self.query.elements,
            },
        }
    }

    pub fn by_attribute<'a>(
        self,
        key: &'a str,
        value: impl Into<QueryValue<'a>>,
    ) -> ElementQuery<'el, 'tree, 'bp, Chain<T, Kind<'a>>> {
        ElementQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::ByAttribute(key, value.into())),
                elements: self.query.elements,
            },
        }
    }

    pub fn first<F, U>(self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>) -> U,
    {
        match self.query(&mut f, false) {
            ControlFlow::Continue(_) => None,
            ControlFlow::Break(val) => Some(val),
        }
    }

    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>),
    {
        _ = self.query(&mut f, true);
    }

    fn query<F, U>(self, f: &mut F, continuous: bool) -> ControlFlow<U>
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>) -> U,
    {
        let ret_val = self.query.elements.children.for_each(|_path, container, children| {
            if let WidgetKind::Element(ref mut element) = container.kind {
                if self.query.filter.filter(element, self.query.elements.attributes) {
                    let attributes = self.query.elements.attributes.get_mut(element.id());
                    let ret_val = f(element, attributes);
                    self.query.elements.dirty_widgets.push(element.id());

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

            let query = ElementQuery {
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
    All,
    ByTag(&'a str),
    ByAttribute(&'a str, QueryValue<'a>),
    AtPosition(Pos),
    ById(WidgetId),
}

impl<'bp, 'a> Filter<'bp> for Kind<'a> {
    type Kind = Element<'bp>;

    fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool {
        match self {
            Kind::All => true,
            Kind::ByTag(tag) => el.ident == *tag,
            Kind::ByAttribute(key, value) => {
                let attribs = attributes.get(el.container.id);
                attribs.get(key).map(|attribute| value.eq(attribute)).unwrap_or(false)
            }
            Kind::AtPosition(pos) => {
                let region = Region::from((el.pos(), el.size()));
                region.contains(*pos)
            }
            Kind::ById(id) => el.id() == *id,
        }
    }
}
