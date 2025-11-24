use anathema_geometry::{Pos, Region, Size};
use unicode_width::UnicodeWidthChar;

use super::renderer::Renderer;
use crate::wgpu::{GraphicsCtx, MaterialId, ScreenConfig, Sprite, State, Style};
use crate::Frontend;

pub struct Ctx<'a> {
    renderer: &'a mut Renderer,
    ctx: &'a mut GraphicsCtx,
    config: &'a ScreenConfig,
}

impl<'a> Ctx<'a> {
    pub(crate) fn new(renderer: &'a mut Renderer, ctx: &'a mut GraphicsCtx, config: &'a ScreenConfig) -> Self {
        Self { renderer, ctx, config }
    }
}

impl<'a> Frontend for Ctx<'a> {
    fn apply_brush_to_region(&mut self, brush: &dyn crate::Brush, region: Region) {
        let style = Style::from(brush);
        // panic!()
        // self.style_region(region, style);
    }

    fn set_text(&mut self, text: &str, pos: Pos) {
        let y = pos.y as usize;
        let mut x = pos.x as usize;
        let mut insertion = self.renderer.back.begin_insert(y);

        for c in text.chars() {
            // Ignore any zero width characters
            let width = match c.width() {
                Some(0) | None => continue,
                Some(w) => w,
            };

            let scale = match self.config {
                &ScreenConfig::CellSize(size) => size,
                _ => unreachable!(),
            };

            // let state = State::Char();
            let mut sprite = self.renderer.font.get_sprite(c, Pos::new(x as f32, y as f32), scale);
            sprite.scale = match self.config {
                ScreenConfig::CellSize(size) => *size,
                ScreenConfig::CellCount { rows, cols } => todo!(),
            };

            let sprite = self.ctx.add_sprite(sprite);

            let state = State::Sprite(sprite);

            // write
            insertion.write_state(x, state);
            if width > 1 {
                insertion.write_state(x + 1, State::Continuation);
            }
            x += width;
        }
    }

    fn invalidate_region(&mut self, region: Region) {
        let start_x = region.from.x as usize;
        let end_x = region.to.x as usize;
        let start_y = region.from.y as usize;
        let end_y = region.to.y as usize;
        self.renderer.back.invalidate_region(start_x, start_y, end_x, end_y);
    }
}
