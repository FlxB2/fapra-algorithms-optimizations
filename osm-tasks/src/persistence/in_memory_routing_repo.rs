#![feature(decl_macro, proc_macro_hygiene)]
use crate::grid_graph::{GridGraph, Node};
use crate::dijkstra::DummyGraph;
use crate::dijkstra::Dijkstra;
use crate::pbf_reader::read_file;
use crate::kml_exporter::KML_export;
use crate::polygon_test::PointInPolygonTest;
use serde::{Deserialize, Serialize};
use std::thread;
use crate::persistence::routing_repo::RoutingRepo;

pub(crate) struct InMemoryRoutingRepo {
    routes: Vec<ShipRoute>,
}

impl RoutingRepo for InMemoryRoutingRepo {
    fn new() -> InMemoryRoutingRepo {
        InMemoryRoutingRepo {
            routes: Vec::new()
        }
    }

    fn add_route(&mut self, route: ShipRoute) -> usize {
        self.routes.push(route);
        self.routes.len()
    }

    fn get_route(&self, id: usize) -> Option<ShipRoute> {
        if self.routes.get(id).is_some() {
            return Some(self.routes[id].clone());
        }
        return None;
    }

    fn has_route(&self, id: usize) -> bool {
        self.routes.len() <= id
    }

    fn get_job_id(&self) -> u32 {
        self.routes.len() as u32
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShipRoute {
    nodes: Vec<Node>,
}

impl ShipRoute {
    pub fn new(nodes: Vec<Node>) -> ShipRoute {
        ShipRoute { nodes }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    file_read: bool,
    polygons_constructed: bool,
    graph_constructed: bool,
    kml_generated: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    start: Node,
    end: Node,
}

impl RouteRequest {
    pub fn start(&self) -> Node {
        self.start
    }
    pub fn end(&self) -> Node {
        self.end
    }
}