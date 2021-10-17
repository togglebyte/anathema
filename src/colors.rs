use std::collections::HashMap;

use super::{panerr, Error, Result};
use pancurses::{init_color, init_pair, COLOR_PAIR};

const THRESHOLD: u8 = 30;

#[derive(Debug, Copy, Clone)]
pub struct Pair(pub(crate) u32);

impl Pair {
    pub fn new(inner: u32) -> Self {
        Self(inner)
    }
}

impl Into<u32> for Pair {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Id(i16),
}

impl From<i16> for Color {
    fn from(n: i16) -> Self {
        match n {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            n => Color::Id(n),
        }
    }
}

impl Into<i16> for Color {
    fn into(self) -> i16 {
        match self {
            Color::Black => 0,
            Color::Red => 1,
            Color::Green => 2,
            Color::Yellow => 3,
            Color::Blue => 4,
            Color::Magenta => 5,
            Color::Cyan => 6,
            Color::White => 7,
            Color::Id(n) => n,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Colors -
// -----------------------------------------------------------------------------
pub struct Colors {
    cache: HashMap<String, i16>,
    next_id: i16,
    max_colors: i16,
}

impl Colors {
    pub fn new() -> Self {
        let max_colors = i16::MAX - 9;
        Self { cache: HashMap::new(), next_id: 9, max_colors }
    }

    pub fn from_hex(&mut self, color: impl AsRef<str>) -> Result<Color> {
        let color = color.as_ref();
        if color.len() != 7 {
            return Err(Error::InvalidColorString(color.into()));
        }

        if !self.cache.contains_key(color) {
            let mut r = u8::from_str_radix(&color[1..=2], 16).unwrap_or(0);
            let mut g = u8::from_str_radix(&color[3..=4], 16).unwrap_or(0);
            let mut b = u8::from_str_radix(&color[5..=6], 16).unwrap_or(0);

            // Make sure the colour isn't weak sauce
            if r < THRESHOLD || g < THRESHOLD || b < THRESHOLD {
                r = r.saturating_add(THRESHOLD);
                g = g.saturating_add(THRESHOLD);
                b = b.saturating_add(THRESHOLD);
            }

            let r = (r as f32 / u8::MAX as f32) * 1000.0;
            let g = (g as f32 / u8::MAX as f32) * 1000.0;
            let b = (b as f32 / u8::MAX as f32) * 1000.0;

            let r = r as i16;
            let g = g as i16;
            let b = b as i16;

            self.init_color(r, g, b)?;
            self.cache.insert(color.into(), self.next_id);

            match self.next_id == self.max_colors {
                true => {
                    self.next_id = 9;
                    // TODO: remove previous colour with this id from the cache
                }
                false => self.next_id += 1,
            }
        }

        let color = self.cache.get(color).map(|id| Color::from(*id)).ok_or(Error::NoColor(color.into()))?;

        Ok(color)
    }

    fn init_color(&self, r: i16, g: i16, b: i16) -> Result<()> {
        let res = init_color(self.next_id, r, g, b);
        panerr!(res, Error::InitColor);
    }

    /// Init pair with BLACK as the background
    pub fn init_fg(color: Color) -> Result<u32> {
        let color_id: i16 = color.into();
        Self::init_pair(color_id, color, Color::Black.into())?;
        Ok(color_id as u32)
    }

    pub fn init_pair(pair_id: i16, fg: Color, bg: Color) -> Result<()> {
        let res = init_pair(pair_id, fg.into(), bg.into());
        panerr!(res, Error::InitPair);
    }

    pub fn get_color_pair(index: u32) -> Pair {
        Pair(COLOR_PAIR(index))
    }
}
