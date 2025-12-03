use std::ops::Range;

use anathema_geometry::{CharacterPos, Region, Size};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::wgpu::{fonts::Char, material::MaterialGroups, Cell, MaterialId, State, Style};

pub struct Buffer {
    size: (usize, usize),
    inner: Vec<Cell>,
    material_regions: Vec<(MaterialId, Region)>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        let (width, height) = (size.width as usize, size.height as usize);
        Self {
            size: (size.width as usize, size.height as usize),
            inner: vec![Cell::default(); width * height],
            material_regions: vec![],
        }
    }

    fn with_region<F>(&self, region: Region, f: F)
    where
        F: Fn(&[Cell]),
    {
        let start_x = region.from.x as usize;
        let end_x = region.to.x as usize;
        let start_y = region.from.y as usize;
        let end_y = region.to.y as usize;

        let (width, _) = self.size;
        for y in start_y..end_y {
            let from = y * width + start_x;
            let to = from + end_x;
            f(&self.inner[from..to]);
        }
    }

    fn with_region_mut<F>(&mut self, region: Region, f: F)
    where
        F: Fn(&mut [Cell]),
    {
        let start_x = region.from.x as usize;
        let end_x = region.to.x as usize;
        let start_y = region.from.y as usize;
        let end_y = region.to.y as usize;

        let (width, _) = self.size;
        for y in start_y..end_y {
            let from = y * width + start_x;
            let to = from + end_x;
            f(&mut self.inner[from..to]);
        }
    }

    pub fn write_style(&mut self, region: Region, style: Style) {
        self.with_region_mut(region, |slice| slice.iter_mut().for_each(|cell| cell.style = style));
    }

    pub(crate) fn clear_region(&mut self, region: Region) {
        self.with_region_mut(region, |slice| slice.fill(Cell::default()));
    }

    pub(crate) fn write_line(&mut self, text: &str, pos: CharacterPos) {
        let from = pos;
        let to = CharacterPos::new(from.x() as i32 + text.width() as i32, from.y() as i32);
        let region = Region::new(from.into(), to.into());
        self.with_region_mut(region, |slice| {
            let mut x = 0;
            for c in text.chars() {
                slice[x].state = State::Char(c);
                x += c.width().unwrap_or(0);
            }
        });
    }

    fn iter_materials(&self, material_groups: &mut MaterialGroups<Char>) {
        for (material, region) in &self.material_regions {
            let mut char_buffer = material_groups.get_or_create(*material);

            self.with_region(*region, |slice| {
                slice;
            });
        }
    }
}
