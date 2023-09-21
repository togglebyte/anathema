use anathema_compiler::*;

fn main() {
    let src = "node [path:  a.-b.c ]";
    let (output, consts) = compile(src).unwrap();
    eprintln!("{output:#?}");

    let val = consts.lookup_value(0.into());

    eprintln!("{val:#?}");
}
