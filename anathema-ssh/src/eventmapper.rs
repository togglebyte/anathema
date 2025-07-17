use anathema_geometry::Size;
use terminput::{
    Event as Terminput_Event, KeyCode as Terminput_KeyCode, KeyEvent as Terminput_KeyEvent,
    KeyEventKind as Terminput_KeyEventKind, KeyModifiers as TermInput_KeyModifiers,
    MouseButton as Terminput_MouseButton, MouseEvent as Terminput_MouseEvent,
    MouseEventKind as Terminput_MouseEventKind, ScrollDirection as Terminput_ScrollDirection,
};

pub fn from_event(value: Terminput_Event) -> Option<anathema_widgets::components::events::Event> {
    match value {
        Terminput_Event::Key(key_event) => from_key_event(key_event),
        Terminput_Event::Mouse(mouse_event) => from_mouse_event(mouse_event),
        Terminput_Event::FocusGained => Some(anathema_widgets::components::events::Event::Focus),
        Terminput_Event::FocusLost => Some(anathema_widgets::components::events::Event::Blur),
        // TODO: Support paste events at some point
        //Terminput_Event::Paste(p) => Some(anathema_widgets::components::events::Event::Paste(p)),
        Terminput_Event::Resize { rows, cols } => Some(anathema_widgets::components::events::Event::Resize(Size {
            width: cols as u16,
            height: rows as u16,
        })),
        _ => None,
    }
}

fn from_key_event(key_event: Terminput_KeyEvent) -> Option<anathema_widgets::components::events::Event> {
    let code = match key_event.code {
        Terminput_KeyCode::Char(c) => anathema_widgets::components::events::KeyCode::Char(c),
        Terminput_KeyCode::Backspace => anathema_widgets::components::events::KeyCode::Backspace,
        Terminput_KeyCode::Enter => anathema_widgets::components::events::KeyCode::Enter,
        Terminput_KeyCode::Left => anathema_widgets::components::events::KeyCode::Left,
        Terminput_KeyCode::Right => anathema_widgets::components::events::KeyCode::Right,
        Terminput_KeyCode::Up => anathema_widgets::components::events::KeyCode::Up,
        Terminput_KeyCode::Down => anathema_widgets::components::events::KeyCode::Down,
        Terminput_KeyCode::Home => anathema_widgets::components::events::KeyCode::Home,
        Terminput_KeyCode::End => anathema_widgets::components::events::KeyCode::End,
        Terminput_KeyCode::PageUp => anathema_widgets::components::events::KeyCode::PageUp,
        Terminput_KeyCode::PageDown => anathema_widgets::components::events::KeyCode::PageDown,
        Terminput_KeyCode::Tab => anathema_widgets::components::events::KeyCode::Tab,
        Terminput_KeyCode::CapsLock => anathema_widgets::components::events::KeyCode::CapsLock,
        Terminput_KeyCode::Delete => anathema_widgets::components::events::KeyCode::Delete,
        Terminput_KeyCode::Insert => anathema_widgets::components::events::KeyCode::Insert,
        Terminput_KeyCode::F(key) => anathema_widgets::components::events::KeyCode::F(key),
        Terminput_KeyCode::Esc => anathema_widgets::components::events::KeyCode::Esc,
        Terminput_KeyCode::KeypadBegin => anathema_widgets::components::events::KeyCode::KeypadBegin,
        _ => return None, // Unsupported key code
    };

    Some(anathema_widgets::components::events::Event::Key(
        anathema_widgets::components::events::KeyEvent {
            code,
            ctrl: key_event.modifiers.contains(TermInput_KeyModifiers::CTRL),
            shift: key_event.modifiers.contains(TermInput_KeyModifiers::SHIFT),
            alt: key_event.modifiers.contains(TermInput_KeyModifiers::ALT),
            super_key: key_event.modifiers.contains(TermInput_KeyModifiers::SUPER),
            hyper: key_event.modifiers.contains(TermInput_KeyModifiers::HYPER),
            meta: key_event.modifiers.contains(TermInput_KeyModifiers::META),
            state: from_key_event_kind(key_event.kind),
        },
    ))
}

fn from_key_event_kind(state: Terminput_KeyEventKind) -> anathema_widgets::components::events::KeyState {
    match state {
        Terminput_KeyEventKind::Press => anathema_widgets::components::events::KeyState::Press,
        Terminput_KeyEventKind::Release => anathema_widgets::components::events::KeyState::Release,
        Terminput_KeyEventKind::Repeat => anathema_widgets::components::events::KeyState::Repeat,
    }
}

fn from_mouse_event(mouse_event: Terminput_MouseEvent) -> Option<anathema_widgets::components::events::Event> {
    let state = match mouse_event.kind {
        Terminput_MouseEventKind::Down(btn) => {
            anathema_widgets::components::events::MouseState::Down(from_mouse_button(btn))
        }
        Terminput_MouseEventKind::Up(btn) => {
            anathema_widgets::components::events::MouseState::Up(from_mouse_button(btn))
        }
        Terminput_MouseEventKind::Drag(btn) => {
            anathema_widgets::components::events::MouseState::Drag(from_mouse_button(btn))
        }
        Terminput_MouseEventKind::Scroll(direction) => from_scroll_direction(direction),
        Terminput_MouseEventKind::Moved => anathema_widgets::components::events::MouseState::Move,
    };

    Some(anathema_widgets::components::events::Event::Mouse(
        anathema_widgets::components::events::MouseEvent {
            x: mouse_event.column,
            y: mouse_event.row,
            state,
        },
    ))
}

fn from_mouse_button(button: Terminput_MouseButton) -> anathema_widgets::components::events::MouseButton {
    match button {
        Terminput_MouseButton::Left => anathema_widgets::components::events::MouseButton::Left,
        Terminput_MouseButton::Right => anathema_widgets::components::events::MouseButton::Right,
        Terminput_MouseButton::Middle => anathema_widgets::components::events::MouseButton::Middle,
        // Unknown buttons are mapped to Left for simplicity
        Terminput_MouseButton::Unknown => anathema_widgets::components::events::MouseButton::Left,
    }
}

fn from_scroll_direction(direction: Terminput_ScrollDirection) -> anathema_widgets::components::events::MouseState {
    match direction {
        Terminput_ScrollDirection::Up => anathema_widgets::components::events::MouseState::ScrollUp,
        Terminput_ScrollDirection::Down => anathema_widgets::components::events::MouseState::ScrollDown,
        Terminput_ScrollDirection::Left => anathema_widgets::components::events::MouseState::ScrollLeft,
        Terminput_ScrollDirection::Right => anathema_widgets::components::events::MouseState::ScrollRight,
    }
}
