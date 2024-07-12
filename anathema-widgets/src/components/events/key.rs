#[derive(Debug, Copy, Clone)]
pub enum KeyState {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub ctrl: bool,
    pub state: KeyState,
}

impl KeyEvent {
    pub fn get_char(&self) -> Option<char> {
        match self.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KeyCode {
    Char(char),
    Tab,
    BackTab,
    CtrlC,
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
}
