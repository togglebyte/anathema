#![feature(test)]
extern crate test;
use anathema::templates::{DataCtx, IncludeCache, NodeCtx, SubContext};
use anathema::widgets::NodeId;
use test::bench::{black_box, Bencher};

#[bench]
fn old(b: &mut Bencher) {
    let src = r#"
    vstack:
        text: "hello world"
        text: "how are you"

        for [data: {{ items }}]:
            text: "value: {{ item }}"
        "#;

    let nodes = anathema::templates::parse(src).unwrap();
    let data = DataCtx::with_value("items", vec![0usize; 60_000]);
    let mut inc_cache = IncludeCache::new();
    let mut node_ctx = NodeCtx::new(&mut inc_cache);
    let sub_ctx = SubContext::new(&data);

    b.iter(|| {
        let nodes = anathema::templates::to_nodes(&nodes, &sub_ctx, &mut node_ctx, NodeId::Auto(0)).unwrap();
        nodes
    });
}
