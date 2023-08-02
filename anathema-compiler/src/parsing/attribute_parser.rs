use anathema_widget_core::{Align, Axis, Direction, Display, TextPath, Value};
use anathema_render::Color;
use anathema_values::{Path, PathId};

use super::fields;
use super::parser::{parse_path, parse_to_fragments};
use crate::error::{ErrorKind, Result};
use crate::lexer::{Kind, Lexer};
use crate::Constants;

const TRUE: &str = "true";
const FALSE: &str = "false";

pub(super) struct AttributeParser<'lexer, 'src> {
    lexer: &'lexer mut Lexer<'src>,
    constants: &'lexer mut Constants,
}

impl<'lexer, 'src> AttributeParser<'lexer, 'src> {
    pub(super) fn new(lexer: &'lexer mut Lexer<'src>, constants: &'lexer mut Constants) -> Self {
        Self { lexer, constants }
    }

    pub(super) fn parse(&mut self, left: &'src str) -> Result<Value> {
        let next = self.lexer.next()?.0;

        match next {
            Kind::String(val) => match parse_to_fragments(val, self.constants) {
                TextPath::String(s) => Ok(Value::String(s)),
                TextPath::Fragments(fragments) => Ok(Value::Fragments(fragments)),
            },
            Kind::Hex(r, g, b) => Ok(Value::Color(Color::Rgb { r, g, b })),
            Kind::Ident(b @ (TRUE | FALSE)) => match b {
                TRUE => Ok(Value::Bool(true)),
                FALSE => Ok(Value::Bool(false)),
                _ => unreachable!(),
            },
            Kind::Ident(val) if val.starts_with("ansi") => match val[4..].parse::<u8>() {
                Ok(ansi_val) => Ok(Value::Color(Color::AnsiValue(ansi_val))),
                Err(_e) => Err(self.lexer.error(ErrorKind::InvalidNumber)),
            },
            Kind::Ident(val) => {
                let val = val.trim();
                match left {
                    fields::ALIGNMENT => match val {
                        "top" => Ok(Value::Alignment(Align::Top)),
                        "top-right" => Ok(Value::Alignment(Align::TopRight)),
                        "right" => Ok(Value::Alignment(Align::Right)),
                        "bottom-right" => Ok(Value::Alignment(Align::BottomRight)),
                        "bottom" => Ok(Value::Alignment(Align::Bottom)),
                        "bottom-left" => Ok(Value::Alignment(Align::BottomLeft)),
                        "left" => Ok(Value::Alignment(Align::Left)),
                        "top-left" => Ok(Value::Alignment(Align::TopLeft)),
                        "centre" | "center" => Ok(Value::Alignment(Align::Centre)),
                        _ => Err(self.lexer.error(ErrorKind::InvalidToken {
                            expected: "alignment",
                        })),
                    },
                    fields::AXIS => match val {
                        "horizontal" | "horz" => Ok(Value::Axis(Axis::Horizontal)),
                        "vertical" | "vert" => Ok(Value::Axis(Axis::Vertical)),
                        _ => Err(self
                            .lexer
                            .error(ErrorKind::InvalidToken { expected: "axis" })),
                    },
                    fields::ID => Ok(Value::String(val.to_string())),
                    fields::DISPLAY => match val {
                        "show" => Ok(Value::Display(Display::Show)),
                        "hide" => Ok(Value::Display(Display::Hide)),
                        "exclude" => Ok(Value::Display(Display::Exclude)),
                        _ => Err(self.lexer.error(ErrorKind::InvalidToken {
                            expected: "display",
                        })),
                    },
                    fields::DIRECTION => match val {
                        "forward" => Ok(Value::Direction(Direction::Forward)),
                        "backward" => Ok(Value::Direction(Direction::Backward)),
                        _ => Err(self
                            .lexer
                            .error(ErrorKind::InvalidToken { expected: "axis" })),
                    },
                    _custom_attribute => match self.try_parse_color(val) {
                        Some(color) => Ok(Value::Color(color)),
                        None => Ok(Value::String(val.to_string())),
                    },
                }
            }
            Kind::Number(val) => Ok(Value::Number(val)),
            Kind::LDoubleCurly => {
                self.lexer.consume(true, false);
                let ident = self.lexer.read_ident()?;
                let path_id = self.try_parse_path(ident)?;
                self.lexer.consume(true, false);
                if !self.lexer.consume_if(Kind::RDoubleCurly)? {
                    return Err(self.lexer.error(ErrorKind::InvalidToken { expected: "}" }));
                }
                Ok(Value::DataBinding(path_id))
            }
            Kind::Colon
            | Kind::Comma
            | Kind::RDoubleCurly
            | Kind::Fullstop
            | Kind::LBracket
            | Kind::RBracket
            | Kind::LParen
            | Kind::RParen
            | Kind::Indent(_)
            | Kind::Newline
            | Kind::Index(_)
            | Kind::Comment
            | Kind::For
            | Kind::In
            | Kind::If
            | Kind::Else
            | Kind::View
            | Kind::EOF => Err(self.lexer.error(ErrorKind::InvalidToken { expected: "" })),
        }
    }

    fn try_parse_path(&mut self, ident: &str) -> Result<PathId> {
        let path = parse_path(self.lexer, ident)?;
        let path_id = self.constants.store_path(path);
        Ok(path_id)
    }

    fn try_parse_color(&mut self, maybe_color: &str) -> Option<Color> {
        match maybe_color {
            "black" => Some(Color::Black),
            "blue" => Some(Color::Blue),
            "cyan" => Some(Color::Cyan),
            "dark-blue" => Some(Color::DarkBlue),
            "dark-cyan" => Some(Color::DarkCyan),
            "dark-green" => Some(Color::DarkGreen),
            "dark-grey" => Some(Color::DarkGrey),
            "dark-magenta" => Some(Color::DarkMagenta),
            "dark-red" => Some(Color::DarkRed),
            "dark-yellow" => Some(Color::DarkYellow),
            "green" => Some(Color::Green),
            "grey" => Some(Color::Grey),
            "magenta" => Some(Color::Magenta),
            "red" => Some(Color::Red),
            "reset" => Some(Color::Reset),
            "white" => Some(Color::White),
            "yellow" => Some(Color::Yellow),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::{Attributes, Fragment};

    use super::*;
    use crate::lexer::Lexer;
    use crate::parsing::parser::{Expression, Parser};
    use crate::parsing::Constants;

    fn parse_attributes(src: &str) -> Attributes {
        parse_attributes_result(src).unwrap()
    }

    fn parse_attributes_result(src: &str) -> Result<Attributes> {
        let mut consts = Constants::default();
        let lexer = Lexer::new(src);
        let parser = Parser::new(lexer, &mut consts)?;
        let mut attrs = Attributes::empty();

        let instructions = parser.collect::<Result<Vec<_>>>()?;
        for inst in instructions {
            match inst {
                Expression::LoadAttribute { key, value } => {
                    let key = consts.strings.get(key).unwrap();
                    let value = consts.values.get(value).unwrap();
                    attrs.set(key, value.clone());
                }
                _ => continue,
            }
        }

        Ok(attrs)
    }

    fn is_true(s: &str, field: &str) -> bool {
        parse_value(s, field).to_bool().unwrap()
    }

    fn parse_num(s: &str, field: &str) -> u64 {
        parse_value(s, field).to_int().unwrap()
    }

    fn parse_value(s: &str, field: &str) -> Value {
        parse_attributes(s).get(field).cloned().unwrap()
    }

    #[test]
    fn parse_height() {
        let height = parse_num("widget [height:1]", fields::HEIGHT);
        assert_eq!(1, height);
    }

    #[test]
    fn parse_width() {
        let width = parse_num("container [width:1]", fields::WIDTH);
        assert_eq!(1, width);
    }

    #[test]
    fn string_fragments() {
        let text = parse_to_fragments("a{{b}}");
        let TextPath::Fragments(fragments) = text else {
            panic!()
        };

        assert_eq!(fragments[0], Fragment::String("a".into()));
        assert_eq!(fragments[1], Fragment::Data(Path::Key("b".to_string())));
    }

    #[test]
    fn escaped_string() {
        let text = parse_to_fragments("a\\\"b");
        let TextPath::String(s) = text else { panic!() };
        assert_eq!(s, "a\"b");
    }

    #[test]
    fn path_key() {
        let mut lexer = Lexer::new(".b.c");
        let path = parse_path(&mut lexer, "a").unwrap();
        assert_eq!("K(a) -> K(b) -> K(c)", path.to_string().as_str());
    }

    #[test]
    fn quoted_attribute() {
        let src = "\"hello, world\"";

        let mut lexer = Lexer::new(src);
        let output = AttributeParser::new(&mut lexer).parse("attrib").unwrap();
        let Value::String(text) = output else {
            panic!()
        };

        assert_eq!(text, "hello, world");
    }

    #[test]
    fn text_attribute() {
        let value = parse_value("widget [value: \"hi\"]", "value");
        assert!(matches!(value, Value::String(_)));
    }

    #[test]
    fn text_fragments_attribute() {
        let value = parse_value("widget [value: \"hi {{ name }} \"]", "value");
        assert!(matches!(value, Value::Fragments(_)));
    }

    #[test]
    fn parse_bool() {
        let is_true = is_true("widget [is_true: true]", "is_true");
        assert!(is_true);
    }

    #[test]
    fn parse_empty_attribs() {
        let attribs = parse_attributes("widget []");
        assert!(attribs.is_empty());
    }

    #[test]
    fn alignment() {
        let align = parse_value("widget [align: top-right]", fields::ALIGNMENT)
            .to_alignment()
            .unwrap();
        assert_eq!(align, Align::TopRight);
    }

    #[test]
    fn parse_colours() {
        let attribs = parse_attributes(
            "widget [background: red, foreground: blue, col: green, res: reset, rgb: #0A0B0C, ansi: ansi123]",
        );

        assert_eq!(
            attribs
                .get(fields::BACKGROUND)
                .and_then(Value::to_color)
                .unwrap(),
            Color::Red
        );

        assert_eq!(
            attribs
                .get(fields::FOREGROUND)
                .and_then(Value::to_color)
                .unwrap(),
            Color::Blue
        );

        assert_eq!(
            attribs.get("col").and_then(Value::to_color).unwrap(),
            Color::Green
        );

        assert_eq!(
            attribs.get("res").and_then(Value::to_color).unwrap(),
            Color::Reset
        );

        assert_eq!(
            attribs.get("rgb").and_then(Value::to_color).unwrap(),
            Color::Rgb {
                r: 10,
                g: 11,
                b: 12
            }
        );

        assert_eq!(
            attribs.get("ansi").and_then(Value::to_color).unwrap(),
            Color::AnsiValue(123)
        );
    }

    #[test]
    fn axis() {
        let dir = parse_value("widget [axis: horz]", fields::AXIS).to_axis();
        assert_eq!(dir.unwrap(), Axis::Horizontal);

        let dir = parse_value("widget [axis: horizontal]", fields::AXIS).to_axis();
        assert_eq!(dir.unwrap(), Axis::Horizontal);

        let dir = parse_value("widget [axis: vert]", fields::AXIS).to_axis();
        assert_eq!(dir.unwrap(), Axis::Vertical);

        let dir = parse_value("widget [axis: vertical]", fields::AXIS).to_axis();
        assert_eq!(dir.unwrap(), Axis::Vertical);
    }

    #[test]
    fn displays() {
        let disp = parse_value("widget [display: show]", fields::DISPLAY).to_display();
        assert_eq!(disp.unwrap(), Display::Show);

        let disp = parse_value("widget [display: hide]", fields::DISPLAY).to_display();
        assert_eq!(disp.unwrap(), Display::Hide);

        let disp = parse_value("widget [display: exclude]", fields::DISPLAY).to_display();
        assert_eq!(disp.unwrap(), Display::Exclude);
    }

    #[test]
    fn whitespace_attribs() {
        // Trim start
        assert!(is_true("text [trim-start: true]", fields::TRIM_START));
        assert!(!is_true("text [trim-start: false]", fields::TRIM_START));

        // // Trim end
        assert!(is_true("text [trim-end: true]", fields::TRIM_END));
        assert!(!is_true("text [trim-end: false]", fields::TRIM_END));

        // // Collapse spaces
        assert!(is_true(
            "text [collapse-spaces: true]",
            fields::COLLAPSE_SPACES
        ));
        assert!(!is_true(
            "text [collapse-spaces: false]",
            fields::COLLAPSE_SPACES
        ));
    }

    #[test]
    fn ansi_color_test() {
        let attribs = parse_attributes("widget [ansi: ansi0]");

        assert_eq!(
            attribs.get("ansi").and_then(Value::to_color).unwrap(),
            Color::AnsiValue(0),
        );
    }

    #[test]
    fn ident_with_pipes() {
        let values = parse_value("widget [meow: a|b|c]", "meow").to_string();
        assert_eq!(values, "a|b|c");
    }

    #[test]
    #[should_panic(expected = "InvalidNumber")]
    fn failed_ansi_color_test() {
        parse_attributes("widget [ansi: ansi256]");
        parse_attributes("widget [ansi: ansi 1]");
    }
}
