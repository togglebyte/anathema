use run::TestCase;
mod run;

#[test]
fn set_attributes() {
    let template = "
test
    test [bold: false, a: 'b']
        ";

    let frame = "
test
    test[bold: Bool(true), a: Str(\"b\")]
        ";

    TestCase::setup(template)
        .build(())
        .with_query(0, |_state, mut elements| {
            elements.by_tag("test").at_position((0, 0)).each(|_el, attribs| {
                attribs.set("bold", true);
            });

            elements.at_position((0, 0)).each(|_el, attribs| {
                assert!(attribs.get::<bool>("bold").unwrap_or(false));
            });

            elements.by_attribute("a", "b").each(|_el, attribs| {
                assert!(attribs.get::<bool>("bold").unwrap_or(false));
            });

            elements.by_tag("test").each(|_el, attribs| {
                assert!(attribs.get::<bool>("bold").unwrap_or(false));
            });
        })
        .expect_frame(frame);
}
