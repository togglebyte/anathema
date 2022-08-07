use crate::display::events::read;
use crate::display::{ScreenPos, Size};
use crate::templates::WidgetNode;

use super::appstate::Receiver;
use super::appstate::Sender;

pub use crate::display::events::{
    CrossEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

/// An event raised by the app state.
pub enum Event<T> {
    /// A keyboard event.
    Key(KeyEvent),
    /// A mouse event.
    Mouse(MouseEvent),
    /// Resize event.
    Resize(Size),
    /// User defined value was sent.
    User(T),
    /// Replace the current widget tree in the [`AppState`].
    ReplaceWidgets(Vec<WidgetNode>),
    /// Terminate the run loop in the app state.
    Quit,
}

impl<T> Event<T> {
    /// Ctrl+c was pressed (useful to terminate the application).
    pub fn ctrl_c(&self) -> bool {
        matches!(self, Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL }))
    }

    /// If the event was a mouse button down event, return
    /// * `ScreenPos`
    /// * `MouseButton`
    /// * `KeyModifiers`
    pub fn mouse_down(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Down(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    /// If the event was a mouse button up event, return
    /// * `ScreenPos`
    /// * `MouseButton`
    /// * `KeyModifiers`
    pub fn mouse_up(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Up(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    /// If the mouse was moved return `(ScreenPos, KeyModifiers)`.
    pub fn mouse_move(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Moved, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    /// If a mouse button was pressed and the mouse moved then this qualifies as a mouse-drag
    /// event.
    /// This returns:
    /// * `ScreenPos`
    /// * `MouseButton`
    /// * `KeyModifiers`
    pub fn mouse_drag(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Drag(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    /// Mouse scroll up, returns `(ScreenPos, KeyModifiers)`.
    pub fn scroll_up(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    /// Mouse scroll down, returns `(ScreenPos, KeyModifiers)`.
    pub fn scroll_down(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    /// If a keyboard character was pressed return the keycode.
    pub fn get_keycode(&self) -> Option<KeyCode> {
        if let Event::Key(KeyEvent { code, .. }) = self {
            Some(*code)
        } else {
            None
        }
    }

    /// If the expected character was pressed return true.
    pub fn is_char(&self, expected: char) -> bool {
        if let Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) = self {
            expected.eq(c)
        } else {
            false
        }
    }

    /// A user defined value was received.
    pub fn user(self) -> Option<T> {
        match self {
            Event::User(val) => Some(val),
            _ => None,
        }
    }
}

pub struct Events<T> {
    event_rx: Receiver<T>,
    tx: Sender<T>,
}

impl<T: Send + Sync + 'static> Events<T> {
    #[cfg(feature = "with-flume")]
    pub fn bounded(cap: usize) -> Self {
        let (tx, rx) = flume::bounded(cap);
        Self::new(tx, rx)
    }

    pub fn unbounded() -> Self {
        #[cfg(feature = "with-flume")]
        let (tx, rx) = flume::unbounded();
        #[cfg(not(feature = "with-flume"))]
        let (tx, rx) = std::sync::mpsc::channel();
        Self::new(tx, rx)
    }

    fn new(event_tx: Sender<T>, event_rx: Receiver<T>) -> Self {
        let tx = event_tx.clone();
        std::thread::spawn(move || events(tx));
        Self { tx: event_tx, event_rx }
    }

    pub fn sender(&self) -> Sender<T> {
        self.tx.clone()
    }

    pub fn next_event(&mut self, blocking: bool) -> Option<Event<T>> {
        match blocking {
            true => self.event_rx.recv().ok(),
            false => self.event_rx.try_recv().ok(),
        }
    }
}

fn events<T>(tx: Sender<T>) {
    while let Ok(event) = read() {
        let payload = match event {
            CrossEvent::Key(e) => Event::Key(e),
            CrossEvent::Mouse(e) => Event::Mouse(e),
            CrossEvent::Resize(w, h) => Event::Resize(Size::new(w as usize, h as usize)),
        };

        let _ = tx.send(payload);
    }
}
