mod string;
use anathema_geometry::Size;
use string::SegString;

fn main() {
    sillybug::init!("/tmp/debbie.sock");

    let string = [("hello", ()), ("x", ()), ("world", ()), (".", ())]
        .into_iter()
        .collect::<SegString<'_, _>>();

    let string = SegString::new("hi. bye sandwich", ());
    let string = SegString::new("superlongword", ());

    for word in string.words() {
        eprintln!("{word}|{}", word.width );
    }

    let mut lines = string.lines(Size::new(5, 10));

    for line in lines {
        let s = line.to_string();
        eprintln!("{s:?}");
    }
}
