#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyState {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn is_ctrl_c(&self) -> bool {
        match self.code {
            KeyCode::Char('c') => self.ctrl,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
