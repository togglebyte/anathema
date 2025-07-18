use std::marker::PhantomData;

use anathema::component::*;

#[derive(Debug, State, Default)]
pub struct BasicState {
    pub boolean: Value<bool>,
    pub number: Value<u32>,
    pub string: Value<String>,
}

pub struct BasicComp<F, T = BasicState>(F, PhantomData<T>);

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

pub fn character(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        ctrl: false,
        shift: false,
        alt: false,
        super_key: false,
        hyper: false,
        meta: false,
        state: KeyState::Press,
    }
}

pub fn tab() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Tab,
        ctrl: false,
        shift: false,
        alt: false,
        super_key: false,
        hyper: false,
        meta: false,
        state: KeyState::Press,
    }
}
