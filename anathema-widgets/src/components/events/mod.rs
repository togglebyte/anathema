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
    Resize(u16, u16),
}
