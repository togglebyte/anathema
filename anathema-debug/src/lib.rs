use std::fmt::{Result, Write};

pub trait DebugWriter {
    fn write(&mut self, output: &mut impl Write) -> Result;
}

pub struct Debug<O>(pub O);

impl<O: Write> Debug<O> {
    pub fn new(output: O) -> Self {
        Self(output)
    }

    pub fn heading(mut self) -> Self {
        let _ = writeln!(&mut self.0, "=== Debug ===");
        self
    }

    pub fn debug(mut self, title: &str, mut item: impl DebugWriter) -> Self {
        let _ = writeln!(&mut self.0, "--- {title} ---");
        let _ = item.write(&mut self.0);
        self
    }

    pub fn footer(mut self) -> Self {
        let _ = writeln!(&mut self.0, "--- EO Debug ---");
        self
    }

    pub fn sep(mut self) -> Self {
        let _ = writeln!(&mut self.0, "----------------");
        self
    }

    pub fn finish(self) -> O {
        self.0
    }
}
