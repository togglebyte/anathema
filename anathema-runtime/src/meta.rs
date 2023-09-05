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
pub struct Meta {
    pub size: Value<Size>,
    pub timings: Value<Timings>,
    pub focus: bool,
    pub count: Value<usize>,
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
pub struct Timings {
    pub layout: Value<Duration>,
    pub position: Value<Duration>,
    pub paint: Value<Duration>,
    pub render: Value<Duration>,
    pub total: Value<Duration>,
}
