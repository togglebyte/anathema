use anathema_render::Color;
use anathema_values::{Path, ScopeValue};

// use anathema_widget_core::{Align, Axis, Direction, Display, Value};
use super::parser::{parse_path, parse_scope_value};
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

    pub(super) fn parse(&mut self, _left: &'src str) -> Result<ScopeValue> {
        let next = self.lexer.next()?.0;

        let value = match next {
            Kind::String(val) => {
                let value = parse_scope_value(val, self.constants);
                return Ok(value);
            }
            // Kind::Hex(r, g, b) => Value::Color(Color::Rgb { r, g, b }),
            // Kind::Ident(b @ (TRUE | FALSE)) => Value::Bool(b == TRUE),
            // Kind::Ident(val) if val.starts_with("ansi") => match val[4..].parse::<u8>() {
            //     Ok(ansi_val) => Value::Color(Color::AnsiValue(ansi_val)),
            //     Err(_e) => return Err(self.lexer.error(ErrorKind::InvalidNumber)),
            // },
            Kind::Number(val) => val,
            Kind::Ident(val) => val.trim(),
            // match left {
            //     fields::ALIGNMENT => match val {
            //         "top" => Value::Alignment(Align::Top),
            //         "top-right" => Value::Alignment(Align::TopRight),
            //         "right" => Value::Alignment(Align::Right),
            //         "bottom-right" => Value::Alignment(Align::BottomRight),
            //         "bottom" => Value::Alignment(Align::Bottom),
            //         "bottom-left" => Value::Alignment(Align::BottomLeft),
            //         "left" => Value::Alignment(Align::Left),
            //         "top-left" => Value::Alignment(Align::TopLeft),
            //         "centre" | "center" => Value::Alignment(Align::Centre),
            //         _ => {
            //             return Err(self.lexer.error(ErrorKind::InvalidToken {
            //                 expected: "alignment",
            //             }))
            //         }
            //     },
            //     fields::AXIS => match val {
            //         "horizontal" | "horz" => Value::Axis(Axis::Horizontal),
            //         "vertical" | "vert" => Value::Axis(Axis::Vertical),
            //         _ => {
            //             return Err(self
            //                 .lexer
            //                 .error(ErrorKind::InvalidToken { expected: "axis" }))
            //         }
            //     },
            //     fields::ID => Value::String(val.to_string()),
            //     fields::DISPLAY => match val {
            //         "show" => Value::Display(Display::Show),
            //         "hide" => Value::Display(Display::Hide),
            //         "exclude" => Value::Display(Display::Exclude),
            //         _ => {
            //             return Err(self.lexer.error(ErrorKind::InvalidToken {
            //                 expected: "display",
            //             }))
            //         }
            //     },
            //     fields::DIRECTION => match val {
            //         "forward" => Value::Direction(Direction::Forward),
            //         "backward" => Value::Direction(Direction::Backward),
            //         _ => {
            //             return Err(self
            //                 .lexer
            //                 .error(ErrorKind::InvalidToken { expected: "axis" }))
            //         }
            //     },
            //     _custom_attribute => match self.try_parse_color(val) {
            //         Some(color) => Value::Color(color),
            //         None => Value::String(val.to_string()),
            //     },
            // }
            // }
            // Kind::Number(val) => Value::Number(val),
            Kind::LDoubleCurly => {
                self.lexer.consume(true, false);
                let ident = self.lexer.read_ident()?;
                let path = self.try_parse_path(ident)?;
                self.lexer.consume(true, false);
                if !self.lexer.consume_if(Kind::RDoubleCurly)? {
                    return Err(self.lexer.error(ErrorKind::InvalidToken { expected: "}" }));
                }
                return Ok(ScopeValue::Dyn(path));
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
            | Kind::EOF => return Err(self.lexer.error(ErrorKind::InvalidToken { expected: "" })),
        };

        Ok(ScopeValue::Static(value.into()))
    }

    fn try_parse_path(&mut self, ident: &str) -> Result<Path> {
        let path = parse_path(self.lexer, ident)?;
        Ok(path)
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
    use std::rc::Rc;

    use anathema_widget_core::generator::Attributes;

    use super::*;
    use crate::lexer::Lexer;
    use crate::parsing::parser::{Expression, Parser};
    use crate::Constants;

    fn parse_attributes(src: &str) -> Attributes {
        parse_attributes_result(src).unwrap()
    }

    fn parse_attributes_result(src: &str) -> Result<Attributes> {
        let mut consts = Constants::new();
        let lexer = Lexer::new(src);
        let parser = Parser::new(lexer, &mut consts)?;
        let mut attrs = Attributes::new();

        let instructions = parser.collect::<Result<Vec<_>>>()?;
        for inst in instructions {
            match inst {
                Expression::LoadAttribute { key, value } => {
                    let key = consts.lookup_string(key).unwrap();
                    let value = consts.lookup_value(value).unwrap();
                    attrs.insert(key.into(), value.clone());
                }
                _ => continue,
            }
        }

        Ok(attrs)
    }

    fn parse_list(s: &str, field: &str) -> Rc<[ScopeValue]> {
        let ScopeValue::List(list) = parse_value(s, field) else {
            panic!()
        };
        list
    }

    fn parse_string(s: &str, field: &str) -> String {
        let ScopeValue::Static(s) = parse_value(s, field) else {
            panic!()
        };
        s.to_string()
    }

    fn parse_value(s: &str, field: &str) -> ScopeValue {
        parse_attributes(s).get(field).cloned().unwrap()
    }

    #[test]
    fn parse_height() {
        let height = parse_string("widget [height:1]", "height");
        assert_eq!("1", height);
    }

    #[test]
    fn parse_width() {
        let width = parse_string("container [width:1]", "width");
        assert_eq!("1", width);
    }

    #[test]
    fn string_fragments() {
        let text = parse_scope_value("a{{b}}", &mut Constants::new());
        let ScopeValue::List(fragments) = text else {
            panic!()
        };

        assert_eq!(fragments[0], ScopeValue::Static("a".into()));
        assert_eq!(fragments[1], ScopeValue::Dyn(Path::Key("b".into())));
    }

    #[test]
    fn escaped_string() {
        let text = parse_scope_value("a\\\"b", &mut Constants::new());
        let ScopeValue::Static(s) = text else {
            panic!()
        };
        assert_eq!(&*s, "a\"b");
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
        let output = AttributeParser::new(&mut lexer, &mut Constants::new())
            .parse("attrib")
            .unwrap();
        let ScopeValue::Static(text) = output else {
            panic!()
        };

        assert_eq!(&*text, "hello, world");
    }

    #[test]
    fn text_attribute() {
        let value = parse_string("widget [value: \"hi\"]", "value");
        assert_eq!(value, "hi");
    }

    #[test]
    fn text_fragments_attribute() {
        let list = parse_list("widget [value: \"hi {{ name }} \"]", "value");
        assert_eq!(list.len(), 3)
    }

    #[test]
    fn parse_bool() {
        let is_true = parse_string("widget [is_true: true]", "is_true");
        assert!(&*is_true == "true");
    }

    #[test]
    fn parse_empty_attribs() {
        let attribs = parse_attributes("widget []");
        assert!(attribs.is_empty());
    }

    #[test]
    fn alignment() {
        let align = parse_string("widget [align: top-right]", "align");
        assert_eq!(&*align, "top-right");
    }

    // #[test]
    // fn parse_colours() {
    //     let attribs = parse_attributes(
    //         "widget [background: red, foreground: blue, col: green, res: reset, rgb: #0A0B0C, ansi: ansi123]",
    //     );

    //     assert_eq!(
    //         attribs
    //             .get(fields::BACKGROUND)
    //             .and_then(Value::to_color)
    //             .unwrap(),
    //         Color::Red
    //     );

    //     assert_eq!(
    //         attribs
    //             .get(fields::FOREGROUND)
    //             .and_then(Value::to_color)
    //             .unwrap(),
    //         Color::Blue
    //     );

    //     assert_eq!(
    //         attribs.get("col").and_then(Value::to_color).unwrap(),
    //         Color::Green
    //     );

    //     assert_eq!(
    //         attribs.get("res").and_then(Value::to_color).unwrap(),
    //         Color::Reset
    //     );

    //     assert_eq!(
    //         attribs.get("rgb").and_then(Value::to_color).unwrap(),
    //         Color::Rgb {
    //             r: 10,
    //             g: 11,
    //             b: 12
    //         }
    //     );

    //     assert_eq!(
    //         attribs.get("ansi").and_then(Value::to_color).unwrap(),
    //         Color::AnsiValue(123)
    //     );
    // }

    #[test]
    fn axis() {
        let dir = parse_string("widget [axis: horz]", "axis");
        assert_eq!(&*dir, "horz");

        let dir = parse_string("widget [axis: horizontal]", "axis");
        assert_eq!(&*dir, "horizontal");

        let dir = parse_string("widget [axis: vert]", "axis");
        assert_eq!(&*dir, "vert");

        let dir = parse_string("widget [axis: vertical]", "axis");
        assert_eq!(&*dir, "vertical");
    }

    #[test]
    fn displays() {
        let disp = parse_string("widget [display: show]", "display");
        assert_eq!(&*disp, "show");

        let disp = parse_string("widget [display: hide]", "display");
        assert_eq!(&*disp, "hide");

        let disp = parse_string("widget [display: exclude]", "display");
        assert_eq!(&*disp, "exclude");
    }

    #[test]
    fn whitespace_attribs() {
        // Trim start
        assert_eq!(
            parse_string("text [trim-start: true]", "trim-start"),
            "true"
        );
        assert_eq!(
            parse_string("text [trim-start: false]", "trim-start"),
            "false"
        );

        // // Trim end
        assert_eq!(parse_string("text [trim-end: true]", "trim-end"), "true");
        assert_eq!(parse_string("text [trim-end: false]", "trim-end"), "false");

        // // Collapse spaces
        assert_eq!(
            parse_string("text [collapse-spaces: true]", "collapse-spaces"),
            "true"
        );

        assert_eq!(
            parse_string("text [collapse-spaces: false]", "collapse-spaces"),
            "false"
        );
    }

    // #[test]
    // fn ansi_color_test() {
    //     let attribs = parse_attributes("widget [ansi: ansi0]");

    //     assert_eq!(
    //         attribs.get("ansi").and_then(Value::to_color).unwrap(),
    //         Color::AnsiValue(0),
    //     );
    // }

    #[test]
    fn ident_with_pipes() {
        let values = parse_string("widget [meow: a|b|c]", "meow");
        assert_eq!(&*values, "a|b|c");
    }

    // #[test]
    // #[should_panic(expected = "InvalidNumber")]
    // fn failed_ansi_color_test() {
    //     parse_attributes("widget [ansi: ansi256]");
    //     parse_attributes("widget [ansi: ansi 1]");
    // }
}
