use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region};
use anathema_state::CommonVal;
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::{apply_visitor, Node, TreeValues};

use crate::nodes::element::Element;
use crate::{AttributeStorage, Attributes, DirtyWidgets, WidgetId, WidgetKind};

// -----------------------------------------------------------------------------
//   - Elements -
// -----------------------------------------------------------------------------
pub struct Elements<'tree, 'bp> {
    nodes: &'tree [Node],
    widgets: &'tree mut TreeValues<WidgetKind<'bp>>,
    attributes: &'tree mut AttributeStorage<'bp>,
    dirty_widgets: &'tree mut DirtyWidgets,
}

impl<'tree, 'bp> Elements<'tree, 'bp> {
    pub fn new(
        nodes: &'tree [Node],
        widgets: &'tree mut TreeValues<WidgetKind<'bp>>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        dirty_widgets: &'tree mut DirtyWidgets,
    ) -> Self {
        Self {
            nodes,
            widgets,
            attributes: attribute_storage,
            dirty_widgets,
        }
    }

    pub fn at_position(&mut self, pos: impl Into<Pos>) -> Query<'_, 'tree, 'bp, Kind<'_>> {
        Query {
            filter: Kind::AtPosition(pos.into()),
            elements: self,
        }
    }

    pub fn by_tag<'tag>(&mut self, tag: &'tag str) -> Query<'_, 'tree, 'bp, Kind<'tag>> {
        Query {
            filter: Kind::ByTag(tag),
            elements: self,
        }
    }

    pub fn by_attribute<'a>(
        &mut self,
        key: &'a str,
        value: impl Into<CommonVal<'a>>,
    ) -> Query<'_, 'tree, 'bp, Kind<'a>> {
        Query {
            filter: Kind::ByAttribute(key, value.into()),
            elements: self,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Query -
// -----------------------------------------------------------------------------
pub struct Query<'el, 'tree, 'bp, F> {
    filter: F,
    elements: &'el mut Elements<'tree, 'bp>,
}

impl<'el, 'tree, 'bp, F> Query<'el, 'tree, 'bp, F>
where
    F: Filter<'bp>,
{
    pub fn by_filter<'a>(self, kind: Kind<'a>) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
        Query {
            filter: FilterChain {
                a: self.filter,
                b: kind,
            },
            elements: self.elements,
        }
    }

    pub fn at_position<'a>(self, pos: impl Into<Pos>) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
        self.by_filter(Kind::AtPosition(pos.into()))
    }

    pub fn by_tag<'a>(self, tag: &'a str) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
        self.by_filter(Kind::ByTag(tag))
    }

    pub fn by_attribute<'a>(
        self,
        key: &'a str,
        value: impl Into<CommonVal<'a>>,
    ) -> Query<'el, 'tree, 'bp, FilterChain<F, Kind<'a>>> {
        self.by_filter(Kind::ByAttribute(key, value.into()))
    }

    fn query(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>), continuous: bool) {
        let mut run = QueryRun {
            filter: self.filter,
            f,
            continuous,
            attributes: self.elements.attributes,
            dirty_widgets: self.elements.dirty_widgets,
        };

        apply_visitor(self.elements.nodes, self.elements.widgets, &mut run);
    }

    pub fn each<T>(self, f: T)
    where
        T: FnMut(&mut Element<'_>, &mut Attributes<'_>),
    {
        self.query(f, true);
    }

    pub fn first(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>)) {
        self.query(f, false);
    }
}

// -----------------------------------------------------------------------------
//   - Query kind -
// -----------------------------------------------------------------------------
pub enum Kind<'a> {
    ByTag(&'a str),
    ByAttribute(&'a str, CommonVal<'a>),
    AtPosition(Pos),
}

impl<'bp, 'a> Filter<'bp> for Kind<'a> {
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
                let region = Region::from((el.container.pos, el.container.size));
                region.contains(*pos)
            }
        }
    }
}

// -----------------------------------------------------------------------------
//   - Filter -
// -----------------------------------------------------------------------------
pub trait Filter<'bp> {
    fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool;

    fn chain(self, other: impl Filter<'bp>) -> impl Filter<'bp>
    where
        Self: Sized,
    {
        FilterChain { a: self, b: other }
    }
}

// -----------------------------------------------------------------------------
//   - Filter chain -
// -----------------------------------------------------------------------------
pub struct FilterChain<A, B> {
    a: A,
    b: B,
}

impl<'a, A, B> FilterChain<A, B>
where
    A: Filter<'a>,
    B: Filter<'a>,
{
    pub fn by_tag(self, tag: &'a str) -> FilterChain<Self, Kind<'a>>
    where
        Self: Sized,
    {
        FilterChain {
            a: self,
            b: Kind::ByTag(tag),
        }
    }

    pub fn by_attribute(self, key: &'a str, value: impl Into<CommonVal<'a>>) -> FilterChain<Self, Kind<'a>>
    where
        Self: Sized,
    {
        FilterChain {
            a: self,
            b: Kind::ByAttribute(key, value.into()),
        }
    }
}

impl<'bp, A: Filter<'bp>, B: Filter<'bp>> Filter<'bp> for FilterChain<A, B> {
    fn filter(&self, el: &Element<'bp>, attributes: &mut AttributeStorage<'_>) -> bool {
        match self.a.filter(el, attributes) {
            false => false,
            true => self.b.filter(el, attributes),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Query runner -
// -----------------------------------------------------------------------------
pub struct QueryRun<'bp, 'tag, T: Filter<'bp>, F> {
    filter: T,
    f: F,
    continuous: bool,
    attributes: &'tag mut AttributeStorage<'bp>,
    dirty_widgets: &'tag mut DirtyWidgets,
}

impl<'bp, 'tag, T: Filter<'bp>, F> NodeVisitor<WidgetKind<'bp>> for QueryRun<'bp, 'tag, T, F>
where
    F: FnMut(&mut Element<'bp>, &mut Attributes<'_>),
{
    fn visit(&mut self, value: &mut WidgetKind<'bp>, _path: &[u16], widget_id: WidgetId) -> ControlFlow<bool> {
        if let WidgetKind::Element(el) = value {
            if self.filter.filter(el, self.attributes) {
                let attributes = self.attributes.get_mut(el.id());
                (self.f)(el, attributes);

                if el.container.inner.any_needs_reflow() {
                    self.dirty_widgets.push(widget_id);
                }

                if !self.continuous {
                    return ControlFlow::Break(false);
                }
            }
        }

        ControlFlow::Continue(())
    }
}
