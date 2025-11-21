use std::ops::Index;

use anathema_geometry::{Pos, Size};
use anathema_hashmap::HashMap;

use crate::wgpu::texture::TextureId;
use crate::wgpu::{MaterialId, Sprite};

pub(crate) static DEFAULT_FONT: &'static [u8] = include_bytes!("font.png");

pub struct Font {
    pub(crate) texture: TextureId,
    pub(crate) texture_size: Size,
    pub(crate) char_size: Size,
    cache: HashMap<char, Sprite>,
}

impl Font {
    pub fn new(texture: TextureId, texture_size: Size, char_size: Size) -> Self {
        Self {
            texture,
            texture_size,
            char_size,
            cache: Default::default(),
        }
    }

    pub(crate) fn get_offset(&self, c: char) -> Pos {
        let pos = match c {
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
        };

        pos * self.char_size
    }

    pub(crate) fn get_sprite(&mut self, c: char, pos: Pos) -> Sprite {
        let offset = self.get_offset(c);
        let sprite = self.cache.entry(c).or_insert_with(|| {
            Sprite::new(
                self.texture,
                MaterialId::default(),
                self.texture_size,
                offset,
                self.char_size,
            )
        });

        sprite.translate(pos * self.char_size * Pos::new(40.0, 40.0));

        sprite.clone()
    }
}
