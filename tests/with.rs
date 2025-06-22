use anathema::component::*;
use anathema::prelude::*;

struct C;

impl Component for C {
    type State = String;
    type Message = ();
}

#[test]
fn eval_with() {
    let tpl = "
    with val as state
        text 'la' val
        ";
    let doc = Document::new("@comp");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);
    builder.component("comp", tpl.to_template(), C, String::from("hello world")).unwrap();
    let res = builder
        .finish(&mut backend, |mut runtime, backend| runtime.run(backend));

    if let Err(e) = res {
        eprintln!("{e}");
    }
}
