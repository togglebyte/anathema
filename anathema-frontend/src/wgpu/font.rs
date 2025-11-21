use std::ops::Index;

use anathema_geometry::Pos;

use crate::wgpu::texture::TextureId;

pub(crate) static DEFAULT_FONT: &'static [u8] = include_bytes!("font.png");

pub struct Font {
    pub(crate) texture: TextureId,
}

impl Font {
    pub fn new(texture: TextureId) -> Self {
        Self { texture }
    }

    pub(crate) fn get(&self, c: char) -> Pos {
        match c {
            ' ' => Pos::ZERO,
            'a' => Pos::new(1.0, 0.0),
            'b' => Pos::new(2.0, 0.0),
            'c' => Pos::new(3.0, 0.0),
            'd' => Pos::new(4.0, 0.0),
            'e' => Pos::new(5.0, 0.0),
            'f' => Pos::new(6.0, 0.0),
            'g' => Pos::new(7.0, 0.0),
            'h' => Pos::new(8.0, 0.0),
            'i' => Pos::new(9.0, 0.0),
            'j' => Pos::new(10.0, 0.0),
            'k' => Pos::new(11.0, 0.0),
            'l' => Pos::new(12.0, 0.0),
            'm' => Pos::new(13.0, 0.0),
            'n' => Pos::new(14.0, 0.0),
            'o' => Pos::new(15.0, 0.0),
            _missing => Pos::new(0.0, 1.0),
        }
    }
}
