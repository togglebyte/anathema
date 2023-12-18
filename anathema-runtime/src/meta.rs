use std::time::Duration;

use anathema_render::Size;

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
