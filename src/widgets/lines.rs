use std::mem;

use unicode_width::UnicodeWidthStr;

use crate::split;

// -----------------------------------------------------------------------------
//     - Instructions -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    String(String),
    Color(u32),
    Pad(usize),
    Reset,
}

impl Instruction {
    fn len(&self) -> usize {
        match self {
            Instruction::String(s) => s.width(),
            Instruction::Pad(size) => *size,
            Instruction::Color(_) => 0,
            Instruction::Reset => 0,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Line -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Line {
    instructions: Vec<Instruction>,
    width: usize,
}

impl Line {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            width: 0,
        }
    }

    pub fn push(&mut self, inst: Instruction) {
        use Instruction::*;
        self.width += match &inst {
            String(s) => s.width(),
            Pad(size) => *size,
            _ => 0
        };

        self.instructions.push(inst);
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn width(&self) -> usize {
        self.width
    }

    fn styles(&self) -> impl Iterator<Item=&Instruction> {
        self.instructions.iter().filter(|i| match i {
            Instruction::Color(_) => true,
            Instruction::Reset => true,
            Instruction::String(_) => false,
            Instruction::Pad(_) => false,
        })
    }
}

impl Default for Line {
    fn default() -> Line {
        Line::new()
    }
}

// -----------------------------------------------------------------------------
//     - Lines -
// -----------------------------------------------------------------------------
pub struct Lines {
    lines: Vec<Line>,
    current_line: Line,
    max_width: usize,
    current_width: usize,
}

impl Lines {
    pub fn new(max_width: usize,) -> Self {
        Self {
            lines: Vec::new(),
            current_line: Line::new(),
            max_width,
            current_width: 0,
        }
    }

    /// Push a string which will in turn be convereted into multiple lines
    /// that fits the given width
    pub fn push_str(&mut self, s: &str) {
        split(s, self.max_width, self.current_width)
            .into_iter()
            .for_each(|line| self.push(Instruction::String(line.to_owned())));
    }

    pub fn push(&mut self, inst: Instruction) {
        // If the current line can't fit the next instruction,
        // insert the current_line into `lines` and create a new
        // `current_line`.
        if self.current_width + inst.len() > self.max_width {
            // Shelve the current line and start a new one

            // Copy any styling from previous line to continue styling the new line.
            let mut current_line = Line::new();
            self.current_line.styles().cloned().for_each(|s| current_line.push(s));

            mem::swap(&mut current_line, &mut self.current_line);
            self.lines.push(current_line);
            self.current_width = 0;
        } 

        self.current_width += inst.len();
        self.current_line.push(inst);
    }

    pub fn complete(mut self) -> Vec<Line> {
        self.lines.push(mem::take(&mut self.current_line));
        self.lines
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn two_lines() {
        let width = 5;
        let input = "123456";
        let mut lines = Lines::new(width);
        lines.push_str(input);
        let lines = lines.complete();
        let expected = Instruction::String("12345".into());
        let actual = &lines[0].instructions()[0];
        assert_eq!(&expected, actual);
    }
}
