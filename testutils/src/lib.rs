use anathema::prelude::*;
use anathema::component::*;

#[derive(Debug, State, Default)]
pub struct Thing {
    pub value: Value<u32>,
}

pub fn char_press(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        ctrl: false,
        state: KeyState::Press,
    }
}
