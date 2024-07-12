use std::time::Duration;

use anathema_widgets::components::events::{Event, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, MouseState};
use crossterm::event::{read, Event as CTEvent, KeyEventKind};
pub use crossterm::event::{
    KeyCode as CTKeyCode, KeyEvent as CTKeyEvent, KeyEventState, KeyModifiers, MouseButton as CTMouseButton,
    MouseEvent as CTMouseEvent, MouseEventKind,
};

/// Event listener
pub struct Events;

impl Events {
    /// Poll events given a duration.
    /// If no event is available within the duration
    /// the function will return `None`.
    pub fn poll(&self, timeout: Duration) -> Option<Event> {
        match crossterm::event::poll(timeout).ok()? {
            true => {
                let event = read().map(Into::into).ok()?;

                let event = match event {
                    CTEvent::Paste(_) => Event::Noop,
                    CTEvent::FocusGained => Event::Focus,
                    CTEvent::FocusLost => Event::Blur,
                    CTEvent::Key(CTKeyEvent {
                        kind: KeyEventKind::Press,
                        code: CTKeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => Event::Stop,
                    CTEvent::Key(key_ev) => Event::Key(key_code_to_key_code(key_ev)),
                    CTEvent::Mouse(mouse_ev) => Event::Mouse(mouse_to_mouse(mouse_ev)),
                    CTEvent::Resize(width, height) => Event::Resize(width, height),
                };

                Some(event)
            }
            false => None,
        }
    }
}

fn key_code_to_key_code(from: CTKeyEvent) -> KeyEvent {
    KeyEvent {
        ctrl: from.modifiers.contains(KeyModifiers::CONTROL),
        code: match from.code {
            CTKeyCode::Backspace => KeyCode::Backspace,
            CTKeyCode::Enter => KeyCode::Enter,
            CTKeyCode::Left => KeyCode::Left,
            CTKeyCode::Right => KeyCode::Right,
            CTKeyCode::Up => KeyCode::Up,
            CTKeyCode::Down => KeyCode::Down,
            CTKeyCode::Home => KeyCode::Home,
            CTKeyCode::End => KeyCode::End,
            CTKeyCode::PageUp => KeyCode::PageUp,
            CTKeyCode::PageDown => KeyCode::PageDown,
            CTKeyCode::Tab => KeyCode::Tab,
            CTKeyCode::BackTab => KeyCode::BackTab,
            CTKeyCode::Delete => KeyCode::Delete,
            CTKeyCode::Insert => KeyCode::Insert,
            CTKeyCode::F(key) => KeyCode::F(key),
            CTKeyCode::Char(c) => KeyCode::Char(c),
            CTKeyCode::Null => KeyCode::Null,
            CTKeyCode::Esc => KeyCode::Esc,
            CTKeyCode::CapsLock => KeyCode::CapsLock,
            CTKeyCode::ScrollLock => KeyCode::ScrollLock,
            CTKeyCode::NumLock => KeyCode::NumLock,
            CTKeyCode::PrintScreen => KeyCode::PrintScreen,
            CTKeyCode::Pause => KeyCode::Pause,
            CTKeyCode::Menu => KeyCode::Menu,
            CTKeyCode::KeypadBegin => KeyCode::KeypadBegin,
            CTKeyCode::Media(_) => KeyCode::Null,
            CTKeyCode::Modifier(_) => KeyCode::Null,
        },
        state: match from.kind {
            KeyEventKind::Press => KeyState::Press,
            KeyEventKind::Repeat => KeyState::Repeat,
            KeyEventKind::Release => KeyState::Release,
        },
    }
}

fn mouse_to_mouse(from: CTMouseEvent) -> MouseEvent {
    MouseEvent {
        x: from.column,
        y: from.row,
        state: match from.kind {
            MouseEventKind::Down(button) => MouseState::Down(button_to_button(button)),
            MouseEventKind::Up(button) => MouseState::Up(button_to_button(button)),
            MouseEventKind::Drag(button) => MouseState::Drag(button_to_button(button)),
            MouseEventKind::Moved => MouseState::Move,
            MouseEventKind::ScrollDown => MouseState::ScrollDown,
            MouseEventKind::ScrollUp => MouseState::ScrollUp,
            MouseEventKind::ScrollLeft => MouseState::ScrollLeft,
            MouseEventKind::ScrollRight => MouseState::ScrollRight,
        },
    }
}

fn button_to_button(button: CTMouseButton) -> MouseButton {
    match button {
        CTMouseButton::Left => MouseButton::Left,
        CTMouseButton::Middle => MouseButton::Middle,
        CTMouseButton::Right => MouseButton::Right,
    }
}
