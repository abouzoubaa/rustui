//! Native model-only graph construction and edit timings, not a rendering benchmark.
use rustui::prelude::{
    CanvasRect, GraphDocument, GraphEdge, GraphEdit, GraphEndpoint, GraphNode, GraphPort,
    GraphPortDirection,
};
use std::{hint::black_box, time::Instant};

fn main() {
    for count in [128, 1_024] {
        let nodes: Vec<_> = (0..count)
            .map(|i| {
                GraphNode::new(
                    format!("node-{i}"),
                    format!("Node {i}"),
                    CanvasRect::new(i as f32, 0.0, 100.0, 50.0).unwrap(),
                    [
                        GraphPort::new("in", "Input", GraphPortDirection::Input).unwrap(),
                        GraphPort::new("out", "Output", GraphPortDirection::Output).unwrap(),
                    ],
                )
                .unwrap()
            })
            .collect();
        let edges: Vec<_> = (0..count - 1)
            .map(|i| {
                GraphEdge::new(
                    format!("edge-{i}"),
                    GraphEndpoint::new(format!("node-{i}"), "out"),
                    GraphEndpoint::new(format!("node-{}", i + 1), "in"),
                )
                .unwrap()
            })
            .collect();
        for sample in 0..5 {
            let start = Instant::now();
            let mut graph = GraphDocument::new(nodes.clone(), edges.clone()).unwrap();
            let construction = start.elapsed();
            let start = Instant::now();
            for i in 0..50 {
                graph = graph
                    .edited(GraphEdit::MoveNode {
                        id: format!("node-{}", i % count),
                        x: i as f32,
                        y: 1.0,
                    })
                    .unwrap();
                black_box(&graph);
            }
            let edits = start.elapsed();
            assert_eq!(graph.nodes().len(), count);
            assert_eq!(graph.edges().len(), count - 1);
            println!(
                "nodes={count} edges={} sample={sample} construction_us={} moves_50_us={}",
                count - 1,
                construction.as_micros(),
                edits.as_micros()
            );
        }
    }
}
