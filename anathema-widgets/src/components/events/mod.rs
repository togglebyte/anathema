use std::time::Duration;

use anathema_geometry::Size;

pub use self::key::{KeyCode, KeyEvent, KeyState};
pub use self::mouse::{MouseButton, MouseEvent, MouseState};

mod key;
mod mouse;

/// An event
#[derive(Debug, Copy, Clone)]
pub enum Event {
    /// No op
    Noop,
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
}
