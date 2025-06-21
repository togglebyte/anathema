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

#[cfg(feature = "filelog")]
pub mod macros {
    #[macro_export]
    macro_rules! debug_to_file {
        ($($arg:tt)*) => {
            use ::std::io::Write as _;
            let mut file = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/log.lol").unwrap();
            let payload = format!($($arg)*);
            file.write_all(payload.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
            file.flush();
        }
    }

    #[macro_export]
    macro_rules! debug_tree {
        ($tree:expr) => {
            let mut d = anathema_widgets::tree::debug::DebugTree::new();
            $tree.apply_visitor(&mut d);
            $crate::debug_to_file!("{}", d.output);
        };
    }
}

#[cfg(not(feature = "filelog"))]
pub mod macros {
    #[macro_export]
    macro_rules! debug_to_file {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! debug_tree {
        ($tree:expr) => {};
    }
}
