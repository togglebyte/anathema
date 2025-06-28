use std::time::Duration;

use anathema_geometry::Size;

pub use self::key::{KeyCode, KeyEvent, KeyState};
pub use self::mouse::{MouseButton, MouseEvent, MouseState};

mod key;
mod mouse;

pub enum EventType {
    PreCycle,
    PostCycle,
}

impl From<Event> for EventType {
    fn from(event: Event) -> Self {
        match event {
            Event::Resize(_) => EventType::PostCycle,
            _ => EventType::PreCycle,
        }
    }
}

/// An event
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    /// No op
    Noop,
    /// Mount, called first time a component is added to the tree
    Mount,
    /// Unmount, called when the component is returned to component storage
    Unmount,
    /// Stop the runtime
    Stop,
    /// Terminal lost focus (not widely supported)
    Blur,
    /// Terminal gained focus (not widely supported)
    Focus,
    /// Key event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Window was resized
    Resize(Size),
    /// Tick
    Tick(Duration),
}

impl Event {
    pub fn is_mouse_event(&self) -> bool {
        matches!(self, Self::Mouse(_))
    }

    pub fn get_char(&self) -> Option<char> {
        match self {
            Self::Key(event) => event.get_char(),
            _ => None,
        }
    }

    pub fn is_ctrl_c(&self) -> bool {
        match self {
            Self::Key(event) => match event.code {
                KeyCode::Char('c') => event.ctrl,
                _ => false,
            },
            _ => false,
        }
    }
}
