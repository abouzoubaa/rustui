//! Model-only comparison of indexed GraphDocument lookup with a linear scan.
//! This does not measure rendering or establish a production performance budget.
use rustui::prelude::{CanvasRect, GraphDocument, GraphNode};
use std::{hint::black_box, time::Instant};

fn main() {
    const NODES: usize = 1_024;
    const ROUNDS: usize = 100;
    let nodes = (0..NODES).map(|i| {
        let id = format!("node-{i}");
        GraphNode::new(
            &id,
            &id,
            CanvasRect::new(i as f32, 0.0, 1.0, 1.0).unwrap(),
            [],
        )
        .unwrap()
    });
    let graph = GraphDocument::new(nodes, []).unwrap();
    let queries: Vec<_> = (0..NODES).map(|i| format!("node-{i}")).collect();

    let run = |indexed: bool| {
        let start = Instant::now();
        let mut found = 0;
        for _ in 0..ROUNDS {
            for id in &queries {
                let id = black_box(id.as_str());
                let node = if indexed {
                    graph.node(id)
                } else {
                    graph.nodes().iter().find(|node| node.id() == id)
                };
                found += usize::from(black_box(node).is_some());
            }
        }
        assert_eq!(found, NODES * ROUNDS);
        start.elapsed()
    };
    // Alternate order to reduce a systematic first-run advantage.
    for sample in 0..6 {
        let (indexed, linear) = if sample % 2 == 0 {
            (run(true), run(false))
        } else {
            let linear = run(false);
            (run(true), linear)
        };
        println!(
            "sample={sample} indexed_us={} linear_us={}",
            indexed.as_micros(),
            linear.as_micros()
        );
    }
}
