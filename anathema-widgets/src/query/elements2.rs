use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region};
use anathema_state::CommonVal;
pub use anathema_store::tree::visitor::apply_visitor;
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::{Node, TreeValues};

use super::{Chain, Filter, Nodes, Query};
use crate::nodes::element::Element;
use crate::{AttributeStorage, Attributes, DirtyWidgets, WidgetContainer, WidgetId, WidgetKind, WidgetTreeView};

pub struct Elements<'tree, 'bp> {
    elements: Nodes<'tree, 'bp>,
}

impl<'tree, 'bp> Elements<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        dirty_widgets: &'tree mut DirtyWidgets,
    ) -> Self {
        Self {
            elements: Nodes::new(children, attribute_storage, dirty_widgets),
        }
    }

    pub fn by_tag<'a>(&mut self, tag: &'a str) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByTag(tag))
    }

    pub fn at_pos<'a>(&mut self, pos: impl Into<Pos>) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::AtPosition(pos.into()))
    }

    pub fn by_attribute<'a>(
        &mut self,
        key: &'a str,
        value: impl Into<CommonVal<'a>>,
    ) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        self.make_query(Kind::ByAttribute(key, value.into()))
    }

    fn make_query<'a>(&mut self, kind: Kind<'a>) -> ElementQuery<'_, 'tree, 'bp, Kind<'a>> {
        ElementQuery {
            query: Query {
                filter: kind,
                elements: &mut self.elements,
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
        value: impl Into<CommonVal<'a>>,
    ) -> ElementQuery<'el, 'tree, 'bp, Chain<T, Kind<'a>>> {
        ElementQuery {
            query: Query {
                filter: Chain::new(self.query.filter, Kind::ByAttribute(key, value.into())),
                elements: self.query.elements,
            },
        }
    }

    pub fn first(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>)) {
        self.query(f, false);
    }

    pub fn each(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>)) {
        self.query(f, true);
    }

    fn query(self, mut f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>), continuous: bool) {
        for i in 0..self.query.elements.children.layout.len() {
            let node = &self.query.elements.children.layout[i];
            let Some((_path, container)) = self.query.elements.children.values.get_mut(node.value()) else {
                continue;
            };

            let WidgetKind::Element(ref mut element) = container.kind else { continue };

            if !self.query.filter.filter(element, self.query.elements.attributes) {
                continue;
            }

            let attributes = self.query.elements.attributes.get_mut(element.id());
            f(element, attributes);

            if !continuous {
                break;
            }

            let mut query = ElementQuery {
                query: Query {
                    elements: self.query.elements,
                    filter: self.query.filter,
                },
            };

            query.query(&mut f, continuous);
        }
    }
}

// -----------------------------------------------------------------------------
//   - Query kind -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum Kind<'a> {
    ByTag(&'a str),
    ByAttribute(&'a str, CommonVal<'a>),
    AtPosition(Pos),
}

impl<'bp, 'a> Filter<'bp> for Kind<'a> {
    type Kind = Element<'bp>;

    fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool {
        match self {
            Kind::ByTag(tag) => el.ident == *tag,
            Kind::ByAttribute(key, value) => {
                let attribs = attributes.get(el.container.id);
                attribs
                    .get_val(key)
                    .and_then(|attribute| {
                        attribute
                            .load_common_val()
                            .and_then(|either| either.to_common().map(|attrib_val| value.eq(&attrib_val)))
                    })
                    .unwrap_or(false)
            }
            Kind::AtPosition(pos) => {
                let region = Region::from((el.container.pos, el.size()));
                region.contains(*pos)
            }
        }
    }
}

// // -----------------------------------------------------------------------------
// //   - Elements -
// // -----------------------------------------------------------------------------
// pub struct Elements<'tree, 'bp> {
//     children: WidgetTreeView<'tree, 'bp>,
//     attributes: &'tree mut AttributeStorage<'bp>,
//     dirty_widgets: &'tree mut DirtyWidgets,
// }

// impl<'tree, 'bp> Elements<'tree, 'bp> {
//     pub fn new(
//         children: WidgetTreeView<'tree, 'bp>,
//         attribute_storage: &'tree mut AttributeStorage<'bp>,
//         dirty_widgets: &'tree mut DirtyWidgets,
//     ) -> Self {
//         Self {
//             children,
//             attributes: attribute_storage,
//             dirty_widgets,
//         }
//     }

//     pub fn at_position(&mut self, pos: impl Into<Pos>) -> Query<'_, 'tree, 'bp, Kind<'_>> {
//         Query {
//             filter: Kind::AtPosition(pos.into()),
//             elements: self,
//         }
//     }

//     pub fn by_tag<'tag>(&mut self, tag: &'tag str) -> Query<'_, 'tree, 'bp, Kind<'tag>> {
//         Query {
//             filter: Kind::ByTag(tag),
//             elements: self,
//         }
//     }

//     pub fn by_attribute<'a>(
//         &mut self,
//         key: &'a str,
//         value: impl Into<CommonVal<'a>>,
//     ) -> Query<'_, 'tree, 'bp, Kind<'a>> {
//         Query {
//             filter: Kind::ByAttribute(key, value.into()),
//             elements: self,
//         }
//     }
// }

// // -----------------------------------------------------------------------------
// //   - Query -
// // -----------------------------------------------------------------------------
// pub struct Query<'el, 'tree, 'bp, F> {
//     filter: F,
//     elements: &'el mut Elements<'tree, 'bp>,
// }

// impl<'el, 'tree, 'bp, F> Query<'el, 'tree, 'bp, F>
// where
//     F: Filter<'bp>,
// {
//     pub fn by_filter<'a>(self, kind: Kind<'a>) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
//         Query {
//             filter: FilterChain {
//                 a: self.filter,
//                 b: kind,
//             },
//             elements: self.elements,
//         }
//     }

//     pub fn at_position<'a>(self, pos: impl Into<Pos>) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
//         self.by_filter(Kind::AtPosition(pos.into()))
//     }

//     pub fn by_tag<'a>(self, tag: &'a str) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
//         self.by_filter(Kind::ByTag(tag))
//     }

//     pub fn by_attribute<'a>(
//         self,
//         key: &'a str,
//         value: impl Into<CommonVal<'a>>,
//     ) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
//         self.by_filter(Kind::ByAttribute(key, value.into()))
//     }

//     fn query(self, mut f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>), continuous: bool) {
//         for i in 0..self.elements.children.layout.len() {
//             // for node in self.elements.children.layout.iter() {
//             let node = &self.elements.children.layout[i];
//             let Some((_path, container)) = self.elements.children.values.get_mut(node.value()) else {
//                 continue;
//             };

//             let WidgetKind::Element(ref mut element) = container.kind else { continue };

//             if !self.filter.filter(element, self.elements.attributes) {
//                 continue;
//             }

//             let attributes = self.elements.attributes.get_mut(element.id());
//             f(element, attributes);

//             let mut query = Query {
//                 filter: self.filter,
//                 elements: self.elements,
//             };

//             if !continuous {
//                 break;
//             }

//             query.query(&mut f, continuous);
//         }
//     }

//     pub fn each<T>(self, f: T)
//     where
//         T: FnMut(&mut Element<'_>, &mut Attributes<'_>),
//     {
//         self.query(f, true);
//     }

//     pub fn first(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>)) {
//         self.query(f, false);
//     }
// }

// // -----------------------------------------------------------------------------
// //   - Filter -
// // -----------------------------------------------------------------------------
// pub trait Filter<'bp>: Copy {
//     fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool;

//     fn chain(self, other: impl Filter<'bp>) -> impl Filter<'bp>
//     where
//         Self: Sized,
//     {
//         FilterChain { a: self, b: other }
//     }
// }

// // -----------------------------------------------------------------------------
// //   - Filter chain -
// // -----------------------------------------------------------------------------
// #[derive(Debug, Copy, Clone)]
// pub struct FilterChain<A, B> {
//     a: A,
//     b: B,
// }

// impl<'a, A, B> FilterChain<A, B>
// where
//     A: Filter<'a>,
//     B: Filter<'a>,
// {
//     pub fn by_tag(self, tag: &'a str) -> FilterChain<Self, Kind<'a>>
//     where
//         Self: Sized,
//     {
//         FilterChain {
//             a: self,
//             b: Kind::ByTag(tag),
//         }
//     }

//     pub fn by_attribute(self, key: &'a str, value: impl Into<CommonVal<'a>>) -> FilterChain<Self, Kind<'a>>
//     where
//         Self: Sized,
//     {
//         FilterChain {
//             a: self,
//             b: Kind::ByAttribute(key, value.into()),
//         }
//     }
// }

// impl<'bp, A: Filter<'bp>, B: Filter<'bp>> Filter<'bp> for FilterChain<A, B> {
//     fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool {
//         match self.a.filter(el, attributes) {
//             false => false,
//             true => self.b.filter(el, attributes),
//         }
//     }
// }
