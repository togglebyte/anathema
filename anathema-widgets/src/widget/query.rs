use std::marker::PhantomData;
use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region};
use anathema_state::{CommonVal, State};
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::{apply_visitor, Node, NodePath, TreeValues};

use crate::nodes::element::Element;
use crate::{AttributeStorage, Attributes, WidgetId, WidgetKind};

pub struct Elements<'tree, 'bp> {
    nodes: &'tree [Node],
    widgets: &'tree mut TreeValues<WidgetKind<'bp>>,
    attributes: &'tree mut AttributeStorage<'bp>,
}

impl<'tree, 'bp> Elements<'tree, 'bp> {
    pub fn new(
        nodes: &'tree [Node],
        widgets: &'tree mut TreeValues<WidgetKind<'bp>>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
    ) -> Self {
        Self {
            nodes,
            widgets,
            attributes: attribute_storage,
        }
    }

    pub fn query<'state, S: State>(&mut self, state: &'state &'state mut S) -> Query<'_, 'tree, 'state, 'bp, S> {
        Query { state, widgets: self }
    }
}

enum QueryArg<'a> {
    ByTag(&'a str),
    ByAttribute(&'a str, CommonVal<'a>),
    AtPosition(Pos),
}

pub struct Query<'widgets, 'tree, 'state, 'bp, S> {
    state: &'state &'state mut S,
    widgets: &'widgets mut Elements<'tree, 'bp>,
}

impl<'widgets, 'tree, 'state, 'bp, S> Query<'widgets, 'tree, 'state, 'bp, S> {
    /// Find elements by its tag (this is the name of the element in the template, e.g `text`,
    /// `vstack` etc.)
    pub fn by_tag(self, ident: &'widgets str) -> QueryResult<'widgets, 'tree, 'state, 'bp, S> {
        QueryResult {
            _s: self.state,
            widgets: self.widgets,
            arg: QueryArg::ByTag(ident),
        }
    }

    /// Find elements based on their attribute values
    pub fn by_attribute(
        self,
        key: &'widgets str,
        value: impl Into<CommonVal<'widgets>>,
    ) -> QueryResult<'widgets, 'tree, 'state, 'bp, S> {
        QueryResult {
            _s: self.state,
            widgets: self.widgets,
            arg: QueryArg::ByAttribute(key, value.into()),
        }
    }

    /// Find elements at a given position
    pub fn at_position(self, pos: impl Into<Pos>) -> QueryResult<'widgets, 'tree, 'state, 'bp, S> {
        let pos = pos.into();
        QueryResult {
            _s: self.state,
            widgets: self.widgets,
            arg: QueryArg::AtPosition(pos),
        }
    }
}

struct QueryRun<'tag, 'bp, F> {
    arg: QueryArg<'tag>,
    p: PhantomData<&'bp ()>,
    f: F,
    continuous: bool,
    attributes: &'tag mut AttributeStorage<'bp>,
}

impl<'tag, 'bp, F> NodeVisitor<WidgetKind<'bp>> for QueryRun<'tag, 'bp, F>
where
    F: FnMut(&mut Element<'bp>, &mut Attributes<'_>),
{
    fn visit(&mut self, value: &mut WidgetKind<'bp>, _path: &NodePath, _widget_id: WidgetId) -> ControlFlow<()> {
        if let WidgetKind::Element(el) = value {
            match self.arg {
                QueryArg::ByTag(tag) if el.ident == tag => {
                    let attributes = self.attributes.get_mut(el.id());
                    (self.f)(el, attributes);
                    if !self.continuous {
                        return ControlFlow::Break(());
                    }
                }
                QueryArg::ByTag(_) => {}
                QueryArg::ByAttribute(key, val) => {
                    let attribs = self.attributes.get(el.container.id);
                    let query_result = attribs
                        .get_val(key)
                        .and_then(|attribute| {
                            attribute
                                .load_common_val()
                                .and_then(|either| either.to_common().map(|attrib_val| val.eq(&*attrib_val)))
                        })
                        .unwrap_or(false);

                    if query_result {
                        let attributes = self.attributes.get_mut(el.id());
                        (self.f)(el, attributes);
                        if !self.continuous {
                            return ControlFlow::Break(());
                        }
                    }
                }
                QueryArg::AtPosition(pos) => {
                    let region = Region::from((el.container.pos, el.container.size));

                    if region.contains(pos) {
                        let attributes = self.attributes.get_mut(el.id());
                        (self.f)(el, attributes);
                        if !self.continuous {
                            return ControlFlow::Break(());
                        }
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }
}

pub struct QueryResult<'widgets, 'tree, 'state, 'bp, S> {
    _s: &'state &'state mut S,
    widgets: &'widgets mut Elements<'tree, 'bp>,
    arg: QueryArg<'widgets>,
}

impl<'widgets, 'tree, 'state, 'bp, S> QueryResult<'widgets, 'tree, 'state, 'bp, S> {
    pub fn each<F>(self, f: F)
    where
        F: FnMut(&mut Element<'_>, &mut Attributes<'_>),
    {
        let mut run = QueryRun {
            arg: self.arg,
            p: PhantomData,
            f,
            continuous: true,
            attributes: self.widgets.attributes,
        };

        apply_visitor(self.widgets.nodes, self.widgets.widgets, &mut run);
    }

    pub fn first(self, f: impl FnMut(&mut Element<'_>, &mut Attributes<'_>)) {
        let mut run = QueryRun {
            arg: self.arg,
            p: PhantomData,
            f,
            continuous: false,
            attributes: self.widgets.attributes,
        };

        apply_visitor(self.widgets.nodes, self.widgets.widgets, &mut run);
    }
}
