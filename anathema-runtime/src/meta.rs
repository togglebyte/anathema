use std::{collections::HashMap, ops::Deref};
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
    pub fn new(size: Size) -> Self {
        Self {
            size: Value::new(size),
            timings: Value::new(Timings::default()),
            focus: true,
            count: Value::new(0),
        }
    }
}

impl State for Meta {
    fn get(&self, key: &Path, node_id: Option<&anathema_values::NodeId>) -> Option<std::borrow::Cow<'_, str>> {
        match key {
            Path::Key(key) => {
                match key.as_str() {
                    "count" => Some((&self.count).into()),
                    _ => None,
                }
            }
            Path::Composite(left, right) => {
                let Path::Key(key) = left.deref() else {
                    return None;
                };
                match key.as_str() {
                    "timings" => self.timings.get(right, node_id),
                    _ => None,
                }
            }
            _ => None
        }
    }

    fn get_collection(&self, key: &Path) -> Option<anathema_values::Collection> {
        None
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

impl State for Timings {
    fn get(&self, key: &Path, node_id: Option<&anathema_values::NodeId>) -> Option<std::borrow::Cow<'_, str>> {
        match key {
            Path::Key(key) => {
                match key.as_str() {
                    "layout" => {
                        if let Some(node_id) = node_id.cloned() {
                            self.layout.subscribe(node_id);
                        }
                        Some(format!("{:?}", self.layout.deref()).into())
                    }
                    "position" => {
                        if let Some(node_id) = node_id.cloned() {
                            self.position.subscribe(node_id);
                        }
                        Some(format!("{:?}", self.position.deref()).into())
                    }
                    "paint" => {
                        if let Some(node_id) = node_id.cloned() {
                            self.paint.subscribe(node_id);
                        }
                        Some(format!("{:?}", self.paint.deref()).into())
                    }
                    "render" => {
                        if let Some(node_id) = node_id.cloned() {
                            self.render.subscribe(node_id);
                        }
                        Some(format!("{:?}", self.render.deref()).into())
                    }
                    "total" => Some(format!("{:?}", self.total.deref()).into()),
                    _ => None,
                }
            }
            _ => None
        }
    }

    fn get_collection(&self, key: &Path) -> Option<anathema_values::Collection> {
        None
    }
}
