use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use anathema_render::Size;
use anathema_values::{Collection, NodeId, Path, State, Value};
use anathema_widget_core::Nodes;

const META: &'static str = "_meta";
const TIMINGS: &'static str = "timings";
const SIZE: &'static str = "size";
const FOCUS: &'static str = "focus";
const COUNT: &'static str = "count";

#[derive(Debug)]
pub struct Meta {
    pub size: Size,
    pub timings: Timings,
    pub focus: bool,
    pub count: usize,
}

impl Meta {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            timings: Timings::default(),
            focus: true,
            count: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Timings {
    pub layout: Duration,
    pub position: Duration,
    pub paint: Duration,
    pub render: Duration,
    pub total: Duration,
}
