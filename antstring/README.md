# AntString (An annotated string)

A string type made up of multiple annotated string slices.

```rust
use antstring::{AntString, Contains, Find};

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

let input = [(&Color::Red, "012"), (&Color::Green, "34"), (&Color::Blue, "5")];
let string = AntString::with_annotations(&input);

assert!(string.contains('3'));
assert_eq!(string.find('3').unwrap(), 3);

for (color, c) in string.annotated_chars() {
    eprintln!("{c} [{color:?}]");
}
```

Thanks to anned20 for the name
