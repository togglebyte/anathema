use anathema::prelude::*;
use anathema_backend::testing::TestBackend;

static TEMPLATE: &str = "
with x as 1 + 2 * 3
    text x
";

#[test]
fn eval_with() {
    let doc = Document::new(TEMPLATE);

    let mut backend = TestBackend::new((10, 3));
    backend.events().next().next().stop();

    let builder = Runtime::builder(doc, &backend);

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    assert_eq!(backend.line(0), "7");

    if let Err(e) = res {
        panic!("{e}");
    }
}
