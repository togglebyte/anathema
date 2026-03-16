mod string;

use anathema_geometry::Size;
use string::SegString;

use crate::string::Index;

fn main() {
    sillybug::init!("/tmp/debbie.sock");

    let string = [("aaaaa", 1), (" x  ", 9), ("bb", 2), ("b", 3), ("bbccccc", 123)]
        .into_iter()
        .collect::<SegString<'_, _>>();

    // let string = [("hello", ()), (" ehthisisalongword", ()), ("world", ()), (".", ())]
    //     .into_iter()
    //     .collect::<SegString<'_, _>>();

    let string = SegString::new("    a b       ", 1);

    // for word in string.words() {
    //     // eprintln!("{word}| {} | {word:#?}", word.width,);
    //     eprintln!("{word} | {}", word.width,);

    //     let (lhs, rhs) = word.split(1);
    //     eprintln!("lhs: {lhs}");
    //     eprintln!("rhs: {rhs}");
    // }

    let mut lines = string.lines(Size::new(5, 10));

    for line in lines {
        let s = line.to_string();
        eprintln!("{s:?}");
    }
}
