use std::collections::VecDeque;

use anathema_widgets::components::events::{Event, KeyEvent};

pub struct EventsMut<'a> {
    event_queue: &'a mut VecDeque<Option<Event>>,
}

impl EventsMut<'_> {
    pub fn next(self) -> Self {
        self.event_queue.push_back(None);
        self
    }

    pub fn next_frames(self, count: usize) -> Self {
        for _ in 0..count {
            self.event_queue.push_back(None);
        }
        self
    }

    pub fn stop(self) -> Self {
        self.event_queue.push_back(Some(Event::Stop));
        self
    }

    pub fn press(self, event: KeyEvent) -> Self {
        self.event_queue.push_back(Some(Event::Key(event)));
        self
    }
}

#[derive(Debug)]
pub struct Events {
    event_queue: VecDeque<Option<Event>>,
}

impl Events {
    pub(super) fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
        }
    }

    pub(super) fn mut_ref(&mut self) -> EventsMut<'_> {
        EventsMut {
            event_queue: &mut self.event_queue,
        }
    }

    pub(super) fn pop(&mut self) -> Option<Event> {
        self.event_queue.pop_front().flatten()
    }
}
