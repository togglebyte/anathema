use std::borrow::Borrow;
use std::ops::Deref;

use anathema_geometry::Size;
use anathema_widgets::components::events::{KeyCode, KeyEvent, KeyState};

use crate::error::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Arg<'src> {
    pub(crate) arg: &'src str,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl<'src> Arg<'src> {
    pub(super) fn new(arg: &'src str, col: usize, line: usize) -> Self {
        Self { arg, col, line }
    }
}

impl<'src> Deref for Arg<'src> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.arg
    }
}

pub(super) fn parse_int(arg: Arg<'_>) -> Result<usize> {
    arg.parse::<usize>().map_err(|_| Error::parse_int(arg))
}

pub(super) fn parse_size(mut args: Vec<Arg<'_>>, line: usize) -> Result<Size> {
    if args.len() != 2 {
        return Err(Error::invalid_num_args(line, args.len()));
    }

    let width = parse_int(args.remove(0))?;
    let height = parse_int(args.remove(0))?;
    Ok(Size::from((width, height)))
}

pub(super) fn parse_key_press(args: Vec<Arg<'_>>, line: usize) -> Result<KeyEvent> {
    let (ctrl, idx) = match args.len() {
        1 => (false, 0),
        2 if args[0].arg == "ctrl" => (true, 1),
        _ => return Err(Error::invalid_num_args(line, 1)),
    };

    Ok(KeyEvent {
        code: str_to_keycode(args[idx])?,
        ctrl,
        state: KeyState::Press,
    })
}

fn str_to_keycode(arg: Arg<'_>) -> Result<KeyCode> {
    let keycode = match arg.arg {
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "del" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "null" => KeyCode::Null,
        "esc" => KeyCode::Esc,
        "capslock" => KeyCode::CapsLock,
        "scrolllock" => KeyCode::ScrollLock,
        "numlock" => KeyCode::NumLock,
        "printscreen" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,
        "menu" => KeyCode::Menu,
        "keypadbegin" => KeyCode::KeypadBegin,
        s if s.chars().count() == 1 => KeyCode::Char(s.chars().next().unwrap()),
        s if s.starts_with("f") && s.chars().count() > 1 => {
            let num = s[1..].parse::<u8>().map_err(|_| Error::parse_int(arg))?;
            KeyCode::F(num)
        },
        _ => return Err(Error::invalid_keycode(arg)),
    };

    Ok(keycode)
}
