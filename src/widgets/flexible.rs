use display::Size;

use crate::attributes::{fields, Attributes};
use crate::Pos;

use super::{Axis, LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// Layout with either tight or loose contraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fit {
    /// Size is the size of the child
    Loose,
    /// Size is the size of the constraints
    Tight,
}

#[derive(Debug)]
pub struct Flexible {
    pub child: Option<WidgetContainer>,
    pub flex_factor: u16,
    pub fit: Fit,
    pub direction: Option<Axis>,
}

impl Flexible {
    pub fn new(flex_factor: u16, fit: Fit, direction: Option<Axis>) -> Self {
        Self {
            child: None,
            flex_factor,
            fit,
            direction,
        }
    }

    fn layout_tight(&mut self, mut ctx: LayoutCtx, direction: Option<Axis>) -> Size {
        match direction {
            Some(Axis::Horizontal) => ctx.constraints.make_width_tight(),
            Some(Axis::Vertical) => ctx.constraints.make_height_tight(),
            None => {
                ctx.constraints.make_width_tight();
                ctx.constraints.make_height_tight();
            }
        }

        if let Some(child) = self.child.as_mut() {
            child.layout(ctx.constraints, ctx.force_layout);
        }

        Size::new(ctx.constraints.min_width, ctx.constraints.min_height)
    }

    fn layout_loose(&mut self, ctx: LayoutCtx) -> Size {
        let size = match self.child.as_mut() {
            Some(child) => child.layout(ctx.constraints, ctx.force_layout),
            None => Size::zero(),
        };

        size
    }
}

impl Widget for Flexible {
    fn kind(&self) -> &'static str {
        "Flexible"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        match self.fit {
            Fit::Loose => self.layout_loose(ctx),
            Fit::Tight => self.layout_tight(ctx, self.direction),
        }
    }

    fn position(&mut self, ctx: PositionCtx) {
        if let Some(child) = self.child.as_mut() {
            child.position(ctx.pos);
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        if let Some(child) = self.child.as_mut() {
            child.paint(ctx.to_unsized());
        }
    }

    fn flex_factor(&self) -> Option<u16> {
        Some(self.flex_factor)
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        match self.child.as_mut() {
            Some(c) => vec![c],
            None => vec![],
        }
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.child = Some(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(ref child) = self.child {
            if child.id.eq(child_id) {
                return self.child.take();
            }
        }
        None
    }

    fn update(&mut self, attributes: Attributes) {
        for (k, _) in &attributes {
            match k.as_str() {
                fields::FIT => self.fit = attributes.fit().unwrap_or(Fit::Tight),
                fields::FLEX => self.flex_factor = attributes.flex().unwrap_or(1),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::layout::Constraints;
    use crate::{Column, Container, Row};

    fn the_test_thing(
        widget: impl Widget,
        fits: Vec<(u16, Fit)>,
    ) -> (WidgetContainer, Vec<NodeId>) {
        let ctx = LayoutCtx::new(Constraints::new(20, 10), false);
        let mut root = widget.into_container(NodeId::auto());

        let ids = fits
            .into_iter()
            .enumerate()
            .map(|(id, (factor, fit))| {
                let mut flex = Flexible::new(factor, fit, None);
                let container = Container::new(None, None);
                let id = NodeId((id as u64).into());
                flex.add_child(container.into_container(id.clone()));
                root.add_child(flex.into_container(NodeId::auto()));

                id
            })
            .collect::<Vec<_>>();

        root.layout(ctx);
        (root, ids)
    }

    #[test]
    fn loose_flex_row() {
        let root = Row::new();
        let (mut root, ids) = the_test_thing(root, vec![(1, Fit::Loose), (1, Fit::Loose)]);

        let top = root.by_id(&ids[0]).unwrap().size.width;
        let bottom = root.by_id(&ids[1]).unwrap().size.width;

        assert_eq!(bottom, 10);
        assert_eq!(top, 10);
    }

    #[test]
    fn tight_flex_col() {
        let root = Column::new();
        let (mut root, ids) = the_test_thing(root, vec![(1, Fit::Tight), (1, Fit::Tight)]);

        let top = root.by_id(&ids[0]).unwrap().size.height;
        let bottom = root.by_id(&ids[1]).unwrap().size.height;

        assert_eq!(top, 5);
        assert_eq!(bottom, 5);
    }

    #[test]
    fn unbalanced_flex_row() {
        let root = Row::new();
        let (mut root, ids) = the_test_thing(root, vec![(3, Fit::Tight), (1, Fit::Tight)]);

        let top = root.by_id(&ids[0]).unwrap().size.width;
        let bottom = root.by_id(&ids[1]).unwrap().size.width;

        assert_eq!(top, 15);
        assert_eq!(bottom, 5);
    }

    #[test]
    fn uneven_flex_row() {
        let root = Row::new();
        let (mut root, ids) = the_test_thing(root, vec![(3, Fit::Tight), (2, Fit::Tight)]);

        let top = root.by_id(&ids[0]).unwrap().size.width;
        let bottom = root.by_id(&ids[1]).unwrap().size.width;

        assert_eq!(top, 12);
        assert_eq!(bottom, 8);
    }

    #[test]
    fn single_flex_col() {
        let root = Column::new();
        let (mut root, ids) = the_test_thing(root, vec![(100, Fit::Tight)]);

        let top = root.by_id(&ids[0]).unwrap();
        assert_eq!(top.size.height, 10);
    }

    // row:
    //  flex [direction: horz, flex: 1]:
    //      container:
    //  flex [direction: horz, flex: 1]:
    //      container:
}
