use std::marker::PhantomData;

use anathema::component::*;

#[derive(Debug, State, Default)]
pub struct BasicState {
    pub number: Value<u32>,
}

pub struct BasicComp<F, T>(F, PhantomData<T>);

impl<F, T> BasicComp<F, T> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<F, T> Component for BasicComp<F, T>
where
    F: FnMut(KeyEvent, &mut T, Children<'_, '_>, Context<'_, '_, T>) + 'static,
    T: State,
{
    type Message = ();
    type State = T;

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        self.0(key, state, children, context);
    }
}

pub fn char_press(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        ctrl: false,
        state: KeyState::Press,
    }
}
