use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::model::grid_graph::Node;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy)]
pub struct BenchmarkResult {
    pub(crate) query_id: usize,
    pub(crate) start_node: Node,
    pub(crate) end_node: Node,
    pub(crate) nmb_nodes: usize,
    pub(crate) distance: u32,
    pub(crate) amount_nodes_popped: u32,
    pub(crate) time: u64,
}

impl BenchmarkResult {
    fn new() -> BenchmarkResult {
        BenchmarkResult {
            query_id: 0,
            start_node: Node::new(),
            end_node: Node::new(),
            nmb_nodes: 0,
            distance: 0,
            amount_nodes_popped: 0,
            time: 0
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AlgoBenchmark {
    pub(crate) results: Vec<BenchmarkResult>,
}

impl AlgoBenchmark {
    pub(crate) fn new() -> AlgoBenchmark {
        AlgoBenchmark {
            results: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct CollectedBenchmarks {
    pub(crate) dijkstra: AlgoBenchmark,
    pub(crate) a_star: AlgoBenchmark,
    pub(crate) bd_dijkstra: AlgoBenchmark,
    pub(crate) ch: AlgoBenchmark,
}
