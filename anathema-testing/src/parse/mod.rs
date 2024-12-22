// It's okay to get an empty value, that might just be an empty string.
// However when `Lines` return None it should be considered fused
use std::collections::HashMap;
use std::str::Lines as Lns;

use anathema_geometry::Size;
use unicode_width::UnicodeWidthStr;

use self::args::Arg;
use crate::error::{Error, Result};
use crate::testcase::{Setup, Step, TestCase};

const DEFAULT_SIZE: Size = Size::new(100, 40);

pub(crate) mod args;

struct Lines<'src> {
    current_line: usize,
    lines: Lns<'src>,
    next: Option<&'src str>,
}

impl<'src> Lines<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            current_line: 0,
            lines: src.lines(),
            next: None,
        }
    }

    fn next(&mut self) -> Option<&'src str> {
        match self.next.take() {
            Some(val) => Some(val),
            None => {
                // TODO: peeking might advance the current line and result in an invalid offset
                self.current_line += 1;
                self.lines.next()
            }
        }
    }

    fn peek(&mut self) -> Option<&'src str> {
        self.next = self.next();
        self.next
    }

    fn next_or<F>(&mut self, f: F) -> Result<&'src str>
    where
        F: Fn(usize) -> Error,
    {
        self.next().ok_or_else(|| f(self.current_line))
    }
}

struct Parser<'src> {
    lines: Lines<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Parser { lines: Lines::new(src) }
    }

    fn parse(&mut self, src: &str) -> Result<TestCase<'src>> {
        let mut lines = src.lines();

        let setup = self.parse_setup()?;
        let steps = self.parse_run()?;

        Ok(TestCase::new(setup, steps))
    }

    fn parse_setup(&mut self) -> Result<Setup<'src>> {
        self.parse_expected_section(Section::Setup)?;

        let mut values = HashMap::new();
        loop {
            if let Some(value) = self.parse_value() {
                values.insert(value.key.arg, value.args);
                continue;
            };

            if let Some("[run]") | None = self.lines.peek() {
                let setup = Setup {
                    title: values
                        .remove("title")
                        .map(|args| args[0].arg),
                    template: values
                        .remove("template")
                        .map(|args| args[0].arg)
                        .ok_or_else(|| Error::missing_key("template", self.lines.current_line))?,
                    size: values
                        .remove("size")
                        .map(|args| args::parse_size(args, self.lines.current_line))
                        .unwrap_or(Ok(DEFAULT_SIZE))?,
                };

                break Ok(setup);
            }
        }
    }

    fn parse_run(&mut self) -> Result<Vec<Step>> {
        self.parse_expected_section(Section::Run)?;

        let mut steps = vec![];

        while let Some(_) = self.lines.peek() {
            if let Some(value) = self.parse_value() {
                let step = value.parse_step()?;
                steps.push(step);
                continue;
            };
        }

        Ok(steps)
    }

    fn parse_expected_section(&mut self, expected: Section) -> Result<()> {
        loop {
            match self.lines.peek().map(str::trim) {
                Some("") | None => {
                    self.lines.next();
                    continue;
                }
                Some(a) => break,
            }
        }

        let section = self.lines.next_or(|line| Error::missing_section(line))?;
        match (expected, section) {
            (Section::Setup, "[setup]") => (),
            (Section::Run, "[run]") => (),
            _ => return Err(Error::missing_section(self.lines.current_line)),
        };

        Ok(())
    }

    fn parse_value(&mut self) -> Option<Value<'src>> {
        let mut value_parser = self
            .lines
            .next()
            .map(|val| ComponentParser::new(val, self.lines.current_line))?;

        let key = value_parser.next()?;
        let args = value_parser.parse_to_vec();

        let value = Value {
            line: self.lines.current_line,
            key,
            args,
        };

        Some(value)
    }
}

#[derive(Debug)]
pub struct Value<'src> {
    line: usize,
    key: Arg<'src>,
    args: Vec<Arg<'src>>,
}

impl Value<'_> {
    fn parse_step(self) -> Result<Step> {
        let step = match self.key.arg {
            "tick" => Step::Tick,
            "press" => Step::KeyPress(args::parse_key_press(self.args, self.line)?),
            "resize" => Step::Resize(args::parse_size(self.args, self.line)?),
            step => return Err(Error::invalid_step(self.line, step))
        };

        Ok(step)
    }
}

// -----------------------------------------------------------------------------
//   - Parse components for values -
//   This will parse both a key and a all the args for a value
// -----------------------------------------------------------------------------
struct ComponentParser<'src> {
    col: usize,
    line: usize,
    src: &'src str,
}

impl<'src> ComponentParser<'src> {
    fn new(src: &'src str, line: usize) -> Self {
        Self { line, col: 0, src }
    }

    fn trim_whitespace(&mut self) {
        let mut trimmed = self.src.trim_start();
        self.col += self.src.width() - trimmed.width();
        self.src = trimmed;
    }

    fn split(&mut self) -> Option<&'src str> {
        let end = self.src.find(char::is_whitespace).unwrap_or(self.src.len());
        let (value, src) = self.src.split_at_checked(end)?;

        if value.is_empty() && src.is_empty() {
            return None;
        }

        self.src = src;
        Some(value)
    }

    fn next(&mut self) -> Option<Arg<'src>> {
        self.trim_whitespace();
        let value = self.split()?;

        let col = self.col;
        self.col += value.width();
        Some(Arg::new(value, col, self.line))
    }

    fn parse_to_vec(mut self) -> Vec<Arg<'src>> {
        let mut values = vec![];

        loop {
            let Some(value) = self.next() else { break };
            values.push(value);
        }

        values
    }
}

enum Section {
    Setup,
    Run,
}

#[cfg(test)]
mod test {
    use anathema_widgets::components::events::{KeyCode, KeyEvent};

    use super::*;

    #[test]
    fn parse_run() {
        let input = "
[setup]
template cookies.aml

[run]
tick

resize 10 5
press a
press ctrl a
            ";
        let mut parser = Parser::new(input);
        let _ = parser.parse_setup().unwrap();
        let steps = parser.parse_run().unwrap();
        assert_eq!(steps[0], Step::Tick);
        assert!(matches!(steps[1], Step::Resize(Size { width: 10, height: 5})));
        assert!(matches!(steps[2], Step::KeyPress(KeyEvent { code: KeyCode::Char('a'), ctrl: false, ..})));
        assert!(matches!(steps[3], Step::KeyPress(KeyEvent { code: KeyCode::Char('a'), ctrl: true, ..})));
    }

    #[test]
    fn parse_setup() {
        let input = "
[setup]
template cookies.aml
size 80 25
            ";
        let mut parser = Parser::new(input);
        let setup = parser.parse_setup().unwrap();
        assert_eq!(setup.template, "cookies.aml");
        assert_eq!(setup.size, Size::new(80, 25));
    }

    #[test]
    fn parse_setup_missing_section() {
        let input = "


key value1 value2
            ";
        let mut parser = Parser::new(input);
        let Err(e) = parser.parse_setup() else { panic!() };
        assert_eq!(e.line, 4);
        assert_eq!(e.col, 1);
    }

    #[test]
    fn parse_value() {
        let input = "abc def";
        let mut parser = ComponentParser::new(input, 0);

        let arg = parser.next().unwrap();
        assert_eq!(arg.arg, "abc");
        assert_eq!(arg.col, 0);

        let arg = parser.next().unwrap();
        assert_eq!(arg.arg, "def");
        assert_eq!(arg.col, 4);
    }

    #[test]
    fn parse_empty() {
        let input = "";
        let mut parser = ComponentParser::new(input, 0);
        assert!(parser.next().is_none());
    }

    #[test]
    fn parse_whitespace_only() {
        let input = "   ";
        let mut parser = ComponentParser::new(input, 0);
        assert!(parser.next().is_none());
    }
}
