use std::collections::HashMap;
use std::time::Duration;

use anathema_render::Size;
use anathema_widget_core::Nodes;
use anathema_values::Path;

use crate::frame::Frame;

const META: &'static str = "_meta";
const TIMINGS: &'static str = "timings";
const SIZE: &'static str = "size";
const FOCUS: &'static str = "focus";
const COUNT: &'static str = "count";

#[derive(Debug)]
pub(super) struct Meta {
    pub(super) size: Size,
    pub(super) timings: Timings,
    pub(super) focus: bool,
    pub(super) count: usize,
}

impl Meta {
    pub(super) fn new(size: Size) -> Self {
        Self {
            size,
            timings: Timings::default(),
            focus: true,
            count: 0,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Timings {
    pub(super) layout: Duration,
    pub(super) position: Duration,
    pub(super) paint: Duration,
    pub(super) render: Duration,
    pub(super) total: Duration,
}
