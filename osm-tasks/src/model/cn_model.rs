use crate::model::grid_graph::{GridGraph, Edge};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CNMetadata {
    pub(crate) graph: GridGraph,
    pub(crate) shortcuts: Vec<Shortcut>,
    pub(crate) is_shortcut: HashMap<String, bool>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub(crate) replaced_edges: Vec<u32>,
    pub(crate) edge: Edge,
}
