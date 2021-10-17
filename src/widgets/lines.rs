use std::mem;
use crate::split;

// -----------------------------------------------------------------------------
//     - Instructions -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Instruction {
    Line(String),
    Color(u32),
    Pad(usize),
    Reset,
}

impl Instruction {
    fn len(&self) -> usize {
        match self {
            Instruction::Line(s) => s.len(),
            Instruction::Pad(size) => *size,
            Instruction::Color(_) => 0,
            Instruction::Reset => 0,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Line -
// -----------------------------------------------------------------------------
pub struct Line {
    instructions: Vec<Instruction>,
    len: usize,
}

impl Line {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, inst: Instruction) {
        use Instruction::*;
        self.len += match &inst {
            Line(s) => s.len(),
            Pad(size) => *size,
            _ => 0
        };

        self.instructions.push(inst);
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    fn styles(&self) -> impl Iterator<Item=&Instruction> {
        self.instructions.iter().filter(|i| match i {
            Instruction::Color(_) => true,
            Instruction::Reset => true,
            Instruction::Line(_) => false,
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

    pub fn push_str(&mut self, s: &str) {
        split(s, self.max_width, self.current_width)
            .into_iter()
            .for_each(|line| self.push(Instruction::Line(line.to_owned())));
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

