use crate::grid_graph::Node;
use std::iter::Map;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy)]
pub struct BenchmarkResult {
    pub(crate) query_id: usize,
    pub(crate) start_node: Node,
    pub(crate) end_node: Node,
    pub(crate) nmb_nodes: usize,
    pub(crate) time: u128,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AlgoBenchmark {
    pub(crate) results: Vec<BenchmarkResult>,

    // TODO: smart formula for this
    pub(crate) avg_time_per_distance: u32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct CollectedBenchmarks {
    pub(crate) results: HashMap<String, AlgoBenchmark>
}
