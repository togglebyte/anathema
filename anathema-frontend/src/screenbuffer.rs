const EMPTY: u32 = u32::MAX;
const CONT: u32 = u32::MAX - 1;

struct Row {
    inner: Vec<u32>,
}

impl Row {
    pub fn new(width: usize) -> Self {
        Self {
            inner: vec![EMPTY; width],
        }
    }
}

struct ScreenBuffer {
    rows: Vec<Row>,
    width: usize,
    height: usize,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            rows: (0..height).map(|_| Row::new(width)).collect(),
            width,
            height,
        }
    }

    fn clear(&mut self) {
        self.rows.iter_mut().for_each(|row| row.inner.clear());
    }
}

pub struct Screen {
    width: usize,
    height: usize,
    back_buffer: ScreenBuffer,
    staging: ScreenBuffer,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            back_buffer: ScreenBuffer::new(width, height),
            staging: ScreenBuffer::new(width, height),
        }
    }

    fn render(&mut self) {
        std::mem::swap(&mut self.back_buffer, &mut self.staging);
        self.staging.clear();
    }
}
