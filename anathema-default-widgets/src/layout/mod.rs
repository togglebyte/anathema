use std::ops::ControlFlow;

use anathema::CommonVal;
use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx};
use anathema_widgets::LayoutChildren;

pub static DIRECTION: &str = "direction";
pub static AXIS: &str = "axis";

pub(crate) mod alignment;
pub(crate) mod border;
pub(crate) mod expand;
pub(crate) mod many;
mod spacers;

pub(crate) fn single_layout<'bp>(
    mut children: LayoutChildren<'_, '_, 'bp>,
    constraints: Constraints,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Size {
    let mut size = Size::ZERO;

    children.for_each(|node, children| {
        size = node.layout(children, constraints, ctx);
        ControlFlow::Break(())
    });

    size
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl TryFrom<CommonVal<'_>> for Axis {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value.to_common_str().as_ref() {
            "horz" | "horizontal" => Ok(Self::Horizontal),
            "vert" | "vertical" => Ok(Self::Vertical),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

impl TryFrom<CommonVal<'_>> for Direction {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value.to_common_str().as_ref() {
            "fwd" | "forward" | "forwards" => Ok(Self::Forward),
            "back" | "backward" | "backwards" => Ok(Self::Backward),
            _ => Err(()),
        }
    }
}
