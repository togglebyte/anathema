use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_value_resolver::ValueKind;
use anathema_widgets::LayoutForEach;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx};

pub static DIRECTION: &str = "direction";
pub static AXIS: &str = "axis";

pub(crate) mod alignment;
pub(crate) mod border;
pub(crate) mod expand;
pub(crate) mod many;
mod spacers;

pub(crate) fn single_layout<'bp>(
    mut children: LayoutForEach<'_, 'bp>,
    constraints: Constraints,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Result<Size> {
    let mut size = Size::ZERO;

    _ = children.each(ctx, |ctx, node, children| {
        size = node.layout(children, constraints, ctx)?.into();
        Ok(ControlFlow::Break(()))
    })?;

    Ok(size)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl TryFrom<&ValueKind<'_>> for Axis {
    type Error = ();

    fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
        let s = value.as_str().ok_or(())?;
        match s {
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

impl TryFrom<&ValueKind<'_>> for Direction {
    type Error = ();

    fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
        let s = value.as_str().ok_or(())?;
        match s {
            "fwd" | "forward" | "forwards" => Ok(Self::Forward),
            "back" | "backward" | "backwards" => Ok(Self::Backward),
            _ => Err(()),
        }
    }
}
