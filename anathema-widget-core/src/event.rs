use std::time::Duration;

use anathema_values::state::State;
use crossterm::event::{read, Event as CTEvent};
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::Nodes;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Noop,
    Quit,
    Blur,
    Focus,
    CtrlC,
    KeyPress(KeyCode, KeyModifiers, KeyEventState),
    KeyRelease(KeyCode, KeyModifiers, KeyEventState),
    KeyRepeat(KeyCode, KeyModifiers, KeyEventState),
    MouseDown(u16, u16, MouseButton, KeyModifiers),
    MouseDrag(u16, u16, MouseButton, KeyModifiers),
    MouseMove(u16, u16, KeyModifiers),
    MouseScrollDown(u16, u16, KeyModifiers),
    MouseScrollMoved(u16, u16, KeyModifiers),
    MouseScrollUp(u16, u16, KeyModifiers),
    MouseScrollLeft(u16, u16, KeyModifiers),
    MouseScrollRight(u16, u16, KeyModifiers),
    MouseUp(u16, u16, MouseButton, KeyModifiers),
    Resize(u16, u16),
}

impl From<CTEvent> for Event {
    fn from(ct_event: CTEvent) -> Self {
        match ct_event {
            CTEvent::Paste(_) => Self::Noop,
            CTEvent::FocusGained => Self::Focus,
            CTEvent::FocusLost => Self::Blur,
            CTEvent::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Self::CtrlC,
            CTEvent::Key(
                ev @ KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                },
            ) => Self::KeyPress(ev.code, ev.modifiers, ev.state),
            CTEvent::Key(
                ev @ KeyEvent {
                    kind: KeyEventKind::Release,
                    ..
                },
            ) => Self::KeyRelease(ev.code, ev.modifiers, ev.state),
            CTEvent::Key(
                ev @ KeyEvent {
                    kind: KeyEventKind::Repeat,
                    ..
                },
            ) => Self::KeyRepeat(ev.code, ev.modifiers, ev.state),
            CTEvent::Mouse(m) => match m.kind {
                MouseEventKind::Down(button) => {
                    Self::MouseDown(m.column, m.row, button, m.modifiers)
                }
                MouseEventKind::Up(button) => Self::MouseUp(m.column, m.row, button, m.modifiers),
                MouseEventKind::Drag(button) => {
                    Self::MouseDrag(m.column, m.row, button, m.modifiers)
                }
                MouseEventKind::Moved => Self::MouseMove(m.column, m.row, m.modifiers),
                MouseEventKind::ScrollDown => Self::MouseScrollDown(m.column, m.row, m.modifiers),
                MouseEventKind::ScrollUp => Self::MouseScrollUp(m.column, m.row, m.modifiers),
                MouseEventKind::ScrollLeft => Self::MouseScrollLeft(m.column, m.row, m.modifiers),
                MouseEventKind::ScrollRight => Self::MouseScrollRight(m.column, m.row, m.modifiers),
            },
            CTEvent::Resize(width, height) => Self::Resize(width, height),
        }
    }
}

pub struct Events;

impl Events {
    pub fn poll(&self, timeout: Duration) -> Option<Event> {
        match crossterm::event::poll(timeout).ok()? {
            true => read().map(Into::into).ok(),
            false => None,
        }
    }
}
