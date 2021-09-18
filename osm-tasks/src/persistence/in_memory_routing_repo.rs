use serde::{Deserialize, Serialize};
use crate::persistence::routing_repo::RoutingRepo;
use crate::model::grid_graph::Node;

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
    distance: u32,
    nodes: Vec<Node>,
}

impl ShipRoute {
    pub fn new(nodes: Vec<Node>, distance: u32) -> ShipRoute {
        ShipRoute { nodes, distance }
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
    pub(crate) start: Node,
    pub(crate) end: Node,
}

impl RouteRequest {
    pub fn start(&self) -> Node {
        self.start
    }
    pub fn end(&self) -> Node {
        self.end
    }
}
