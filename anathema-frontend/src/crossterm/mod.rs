use crate::screenbuffer::Screen;

pub struct Render {
    screen: Screen,
}

impl Render {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            screen: Screen::new(width, height),
        }
    }

    pub fn render(&self) {
        // let _ = execute!(&mut self.output, BeginSynchronizedUpdate);
        // let _ = self.screen.render(&mut self.output, glyph_map);
        // let _ = execute!(&mut self.output, EndSynchronizedUpdate);
    }
}
