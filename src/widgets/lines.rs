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
        Self { instructions: Vec::new(), width: 0 }
    }

    fn push(&mut self, inst: Instruction) {
        use Instruction::*;
        self.width += match &inst {
            String(s) => s.width(),
            Pad(size) => *size,
            _ => 0,
        };

        self.instructions.push(inst);
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn width(&self) -> usize {
        self.width
    }

    fn styles(&self) -> impl Iterator<Item = &Instruction> {
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
    max_width: usize,
    current_width: usize,
    start_newline: bool,
}

impl Lines {
    pub fn new(max_width: usize) -> Self {
        Self { 
            lines: Vec::new(),
            max_width,
            current_width: 0,
            start_newline: false 
        }
    }

    fn current_line(&mut self) -> &mut Line {
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
        self.lines.last_mut().expect("well this is a surprise")
    }

    /// Push a string which will in turn be convereted into multiple lines
    /// that fits the given width
    pub fn push_str(&mut self, s: &str, keep_whitespace: bool) {
        for line in split(s, self.max_width, self.current_width, keep_whitespace) {
            self.push(Instruction::String(line.into()));
        }
    }

    /// Pad a line with space
    pub fn pad(&mut self, pad: usize) {
        self.push(Instruction::Pad(pad));
    }

    /// Set a color
    pub fn color(&mut self, color: u32) {
        self.push(Instruction::Color(color));
    }

    /// Reset the colors (set color to zero)
    pub fn reset(&mut self) {
        self.push(Instruction::Reset);
    }

    fn push(&mut self, inst: Instruction) {
        // If the current line can't fit the next instruction,
        // insert the current_line into `lines` and create a new
        // `current_line`.
        if self.current_width + inst.len() > self.max_width || self.start_newline {
            // Shelve the current line and start a new one

            // Copy any styling from previous line to continue styling the new line.
            let mut current_line = Line::new();
            self.current_line().styles().cloned().for_each(|s| current_line.push(s));
            self.lines.push(current_line);

            self.current_width = 0;
            self.start_newline = false;
        }

        if matches!(inst, Instruction::String(ref s) if s.ends_with('\n')) {
            self.start_newline = true;
        }

        self.current_width += inst.len();
        self.current_line().push(inst);
    }

    pub fn iter(&self) -> impl Iterator<Item=&Line> {
        self.lines.iter()
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, Line> {
        self.lines.drain(..)
    }

    /// Number of lines
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn resize(&mut self, new_max_width: usize) {
        self.max_width = new_max_width;

        let mut lines = Lines::new(new_max_width);

        self.lines
            .drain(..)
            .flat_map(|line| line.instructions)
            .for_each(|instruction| lines.push(instruction));

        *self = lines;
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
        lines.push_str(input, false);
        let lines = lines.iter().collect::<Vec<&Line>>();
        let expected = Instruction::String("12345".into());
        let actual = &lines[0].instructions()[0];
        assert_eq!(&expected, actual);
    }

    #[test]
    fn test_three_lines() {
        let width = 4;
        let input = "123456789";
        let mut lines = Lines::new(width);
        lines.push_str(input, false);
        let lines = lines.iter();
        let expected = 3;

        let actual = lines.count();
        assert_eq!(expected, actual);
    }

    #[test]
    fn resize() {
        let width = 4;
        let input = "123456789";
        let mut lines = Lines::new(width);
        lines.push_str(input, false);
        lines.resize(5);
        let expected = 2;
        let actual = lines.len();
        assert_eq!(expected, actual);
    }

    #[test]
    fn resize_with_newlines() {
        let width = 4;
        let input = "1234\n5678\n9";
        let mut lines = Lines::new(width);
        lines.push_str(input, false);
        lines.resize(5);
        let expected = 3;
        let actual = lines.len();
        assert_eq!(expected, actual);
    }

    #[test]
    fn styles_spans_multiplie_lines() {
        let width = 1;
        let input = "ab";
        let mut lines = Lines::new(width);
        lines.push(Instruction::Color(5));
        lines.push_str(input, false);

        let second_line = &lines.iter().collect::<Vec<_>>()[1];
        assert!(matches!(second_line.instructions()[0], Instruction::Color(5)));

        let s = &second_line.instructions()[1];

        match s {
            Instruction::String(s) if s == "b" => {}
            _ => panic!("wrong wrong wrong"),
        }
    }

    #[test]
    fn split_lines_on_word_boundary() {
        let line = r#"    let y = "this is a longer string that should have lots of wonderful spelling mistakes and what not in it and we have to make sure this spans multiple lines to see if it works";"#;
        // let line = "     hello world";
        // let max_width = 13;
        let max_width = 125;
        let mut lines = Lines::new(max_width);
        lines.push_str(line, true);

        for line in lines.iter() {
            eprintln!("{:?}", line);
        }
    }
}
