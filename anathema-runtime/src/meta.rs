use std::collections::HashMap;
use std::time::Duration;

use anathema_render::Size;
use anathema_widget_core::Nodes;
use anathema_values::{Path, Value, State};

use crate::frame::Frame;

const META: &'static str = "_meta";
const TIMINGS: &'static str = "timings";
const SIZE: &'static str = "size";
const FOCUS: &'static str = "focus";
const COUNT: &'static str = "count";

#[derive(Debug)]
pub(super) struct Meta {
    pub(super) size: Value<Size>,
    pub(super) timings: Value<Timings>,
    pub(super) focus: bool,
    pub(super) count: Value<usize>,
}

impl Meta {
    pub(super) fn new(size: Size) -> Self {
        Self {
            size: Value::new(size),
            timings: Value::new(Timings::default()),
            focus: true,
            count: Value::new(0),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Timings {
    pub(super) layout: Value<Duration>,
    pub(super) position: Value<Duration>,
    pub(super) paint: Value<Duration>,
    pub(super) render: Value<Duration>,
    pub(super) total: Value<Duration>,
}
