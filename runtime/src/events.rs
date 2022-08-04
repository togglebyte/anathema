use display::events::read;
pub use display::events::{CrossEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use display::{ScreenPos, Size};

use crate::appstate::Receiver;
use crate::appstate::Sender;

pub enum Event<T> {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(Size),
    User(T),
    Quit,
}

impl<T> Event<T> {
    pub fn ctrl_c(&self) -> bool {
        matches!(self, Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL }))
    }

    pub fn scroll_up(&self) -> bool {
        matches!(self, Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, .. }))
    }

    pub fn scroll_down(&self) -> bool {
        matches!(self, Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, .. }))
    }

    pub fn mouse_down(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Down(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    pub fn mouse_up(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Up(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    pub fn mouse_move(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Moved, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    pub fn mouse_drag(&self) -> Option<(ScreenPos, MouseButton, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::Drag(btn), column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *btn, *modifiers))
            }
            _ => None,
        }
    }

    pub fn mouse_scroll_up(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    pub fn mouse_scroll_down(&self) -> Option<(ScreenPos, KeyModifiers)> {
        match self {
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column, row, modifiers }) => {
                Some((ScreenPos::new(*column, *row), *modifiers))
            }
            _ => None,
        }
    }

    pub fn is_char(&self, expected: char) -> bool {
        if let Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) = self {
            expected.eq(c)
        } else {
            false
        }
    }

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

    pub fn tx(&self) -> Sender<T> {
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
