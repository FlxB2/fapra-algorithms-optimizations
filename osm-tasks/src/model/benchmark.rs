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

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AlgoBenchmark {
    pub(crate) results: Vec<BenchmarkResult>,
    pub(crate) avg_distance_per_ms: f32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct CollectedBenchmarks {
    pub(crate) results: HashMap<String, AlgoBenchmark>
}
