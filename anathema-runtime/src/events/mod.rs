use std::time::Duration;

use anathema_values::state::State;
use anathema_widget_core::nodes::Nodes;
use crossterm::event::{read, Event as CTEvent};
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEventKind,
};

// use crate::meta::Meta;

pub mod flume;

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

pub trait Events<S: State> {
    fn event(&mut self, event: Event, tree: &mut Nodes, state: &mut S) -> Event;
}

pub struct DefaultEvents<F, S>(pub F, pub std::marker::PhantomData<S>)
where
    F: FnMut(Event, &mut Nodes, &mut S) -> Event;

impl<F, S: State> Events<S> for DefaultEvents<F, S>
where
    F: FnMut(Event, &mut Nodes, &mut S) -> Event,
{
    fn event(&mut self, event: Event, tree: &mut Nodes, state: &mut S) -> Event {
        (self.0)(event, tree, state)
    }
}

/// The event receiver should batch events to prevent starving the event loop.
pub trait EventProvider {
    fn next(&mut self) -> Option<Event>;
}

/// Default event provider, blocks the runtime for N ms
pub struct DefaultEventProvider(Duration);

impl DefaultEventProvider {
    pub fn with_timeout(timeout: u64) -> Self {
        Self(Duration::from_millis(timeout))
    }
}

impl EventProvider for DefaultEventProvider {
    fn next(&mut self) -> Option<Event> {
        match crossterm::event::poll(self.0).ok()? {
            true => read().map(Into::into).ok(),
            false => None,
        }
    }
}
