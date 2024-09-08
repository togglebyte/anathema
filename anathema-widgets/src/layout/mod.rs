use std::ops::ControlFlow;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{Node, TreeFilter, TreeForEach, TreeValues};

pub use self::constraints::Constraints;
pub use self::display::Display;
use crate::nodes::element::Element;
use crate::{AttributeStorage, WidgetId, WidgetKind};

mod constraints;
mod display;
pub mod text;

#[derive(Debug, Copy, Clone)]
/// A viewport represents the available space in the root
pub struct Viewport {
    size: Size,
}

impl Viewport {
    pub fn new(size: impl Into<Size>) -> Self {
        Self { size: size.into() }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn constraints(&self) -> Constraints {
        Constraints::new(self.size.width, self.size.height)
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
    }
}

/// Filter out widgets that are excluded.
/// This includes both `Show` and `Hide` as part of the layout.
pub struct LayoutFilter<'frame, 'bp> {
    attributes: &'frame AttributeStorage<'bp>,
    ignore_floats: bool,
}

impl<'frame, 'bp> LayoutFilter<'frame, 'bp> {
    pub fn new(ignore_floats: bool, attributes: &'frame AttributeStorage<'bp>) -> Self {
        Self {
            attributes,
            ignore_floats,
        }
    }
}

impl<'frame, 'bp> TreeFilter for LayoutFilter<'frame, 'bp> {
    type Input = WidgetKind<'bp>;
    type Output = Element<'bp>;

    fn filter<'val>(
        &self,
        _value_id: WidgetId,
        input: &'val mut Self::Input,
        children: &[Node],
        widgets: &mut TreeValues<WidgetKind<'bp>>,
    ) -> ControlFlow<(), Option<&'val mut Self::Output>> {
        match input {
            WidgetKind::Element(el) if el.container.inner.any_floats() && self.ignore_floats => ControlFlow::Break(()),
            WidgetKind::Element(el) => match self
                .attributes
                .get(el.id())
                .get::<Display>("display")
                .unwrap_or_default()
            {
                Display::Show | Display::Hide => ControlFlow::Continue(Some(el)),
                Display::Exclude => ControlFlow::Continue(None),
            },
            WidgetKind::ControlFlow(widget) => {
                // TODO `update` should probably be called `layout`
                //       as it does not update during an update step.
                //
                //       That is not possible since the child widget is
                //       checked out already, so iterating over the children
                //       of ControlFlow does not work
                widget.update(children, widgets);
                ControlFlow::Continue(None)
            }
            WidgetKind::If(widget) if !widget.show => ControlFlow::Break(()),
            WidgetKind::Else(widget) if !widget.show => ControlFlow::Break(()),
            _ => ControlFlow::Continue(None),
        }
    }
}

pub struct LayoutCtx<'a, 'bp> {
    pub attribs: &'a AttributeStorage<'bp>,
    pub viewport: &'a Viewport,
}

impl<'a, 'bp> LayoutCtx<'a, 'bp> {
    pub fn new(attribs: &'a AttributeStorage<'bp>, viewport: &'a Viewport) -> Self {
        Self { attribs, viewport }
    }
}

pub fn layout_widget<'bp>(
    element: &mut Element<'bp>,
    children: &[Node],
    values: &mut TreeValues<WidgetKind<'bp>>,
    constraints: Constraints,
    ctx: &mut LayoutCtx<'_, 'bp>,
    ignore_floats: bool,
) {
    let filter = LayoutFilter::new(ignore_floats, ctx.attribs);
    let children = TreeForEach::new(children, values, &filter);
    element.layout(children, constraints, ctx);
}

pub fn position_widget<'bp>(
    pos: Pos,
    element: &mut Element<'bp>,
    children: &[Node],
    values: &mut TreeValues<WidgetKind<'bp>>,
    attribute_storage: &AttributeStorage<'bp>,
    ignore_floats: bool,
    viewport: Viewport,
) {
    let filter = LayoutFilter::new(ignore_floats, attribute_storage);
    let children = TreeForEach::new(children, values, &filter);
    element.position(children, pos, attribute_storage, viewport);
}

#[derive(Debug, Copy, Clone)]
pub struct PositionCtx {
    pub inner_size: Size,
    pub pos: Pos,
    pub viewport: Viewport,
}

#[cfg(test)]
mod test {
    use anathema_geometry::Region;

    use super::*;

    #[test]
    fn region_inersect() {
        let a = Region::new(Pos::ZERO, Pos::new(10, 10));
        let b = Region::new(Pos::new(5, 5), Pos::new(8, 8));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn region_contains() {
        let a = Region::new(Pos::ZERO, Pos::new(10, 10));
        assert!(a.contains(Pos::ZERO));
        assert!(a.contains(Pos::new(9, 9)));
        assert!(!a.contains(Pos::new(10, 10)));
    }
}
