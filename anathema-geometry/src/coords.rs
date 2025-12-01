use std::marker::PhantomData;

use crate::{Pos as InnerPos, Size};

pub type CharacterPos = Pos<Character>;
pub type ScreenPos = Pos<Screen>;

/// Screen coordinates refers to the coordinate space of the active monitor.
/// `0,0` is the top left of the monitor.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Screen;

impl Pos<Screen> {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            coord_sys: PhantomData,
            inner: InnerPos::new(x as f32, y as f32),
        }
    }

    pub fn to_char_pos(self, char_size: Size) -> CharacterPos {
        let inner = (self.inner / char_size).trunc().into();
        CharacterPos {
            coord_sys: PhantomData,
            inner,
        }
    }
}

/// The coordinate space of the window.
/// `0,0` is the top left of the window.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Window;

/// The coordinate space of the "world".
/// `0,0` is the centre of the world.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct World;

/// The coordinate space of the UI layer.
/// `0,0` is the same as
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Character;

impl Pos<Character> {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            coord_sys: PhantomData,
            inner: InnerPos::new(x as f32, y as f32),
        }
    }

    pub fn to_screen_pos(self, char_size: Size) -> Pos<Screen> {
        let inner = (self.inner * char_size).trunc().into();
        ScreenPos {
            coord_sys: PhantomData,
            inner,
        }
    }

    pub fn to_usize(self) -> (usize, usize) {
        (self.inner.x as usize, self.inner.y as usize)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pos<C> {
    coord_sys: PhantomData<C>,
    inner: InnerPos,
}

impl<C> Pos<C> {
    const ZERO: Self = Self {
        coord_sys: PhantomData,
        inner: InnerPos::ZERO,
    };
}

impl<T> From<Pos<T>> for InnerPos {
    fn from(value: Pos<T>) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn screen_to_char() {
        let char_size = Size::new(16.4, 32.2);

        let screen = ScreenPos::new(16, 1);

        let expected = CharacterPos::ZERO;
        assert_eq!(expected, screen.to_char_pos(char_size));

        let screen = ScreenPos::new(17, 65);

        let expected = CharacterPos::new(1, 2);
        assert_eq!(expected, screen.to_char_pos(char_size));
    }

    #[test]
    fn char_to_screen() {
        let char_size = Size::new(16.0, 32.0);
        let character = CharacterPos::new(2, 3);

        let expected = ScreenPos::new(16 * 2, 32 * 3);
        assert_eq!(expected, character.to_screen_pos(char_size));

        let expected = ScreenPos::ZERO;
        assert_eq!(expected, CharacterPos::ZERO.to_screen_pos(char_size));
    }
}
