use anathema_compiler::*;

fn main() {
    let src = "node [padding:  [1 2 3 4]]";
    let (output, consts) = compile(src).unwrap();
    eprintln!("{output:#?}");

    let val = consts.lookup_value(0.into());

    eprintln!("{val:#?}");
}
