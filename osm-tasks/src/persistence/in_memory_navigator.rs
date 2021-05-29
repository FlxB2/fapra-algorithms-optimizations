use crate::grid_graph::{GridGraph, Node};
use crate::pbf_reader::{read_file, read_or_create_graph};
use crate::polygon_test::PointInPolygonTest;
use crate::persistence::navigator::Navigator;
use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use std::sync::Mutex;
use crate::dijkstra::{Dijkstra};
use crate::nearest_neighbor::NearestNeighbor;

pub(crate) struct InMemoryGraph {
    graph: GridGraph,
    dijkstra: Option<Dijkstra>,
    nearest_neighbor: Option<NearestNeighbor>
}

impl Navigator for InMemoryGraph {
    fn new() -> InMemoryGraph {
        InMemoryGraph {
            graph: GridGraph::default(),
            dijkstra: None,
            nearest_neighbor: None
        }
    }

    fn build_graph(&mut self) {
        /*let polygons =
        //let polygons = read_file("./iceland-coastlines.osm.pbf");
        let polygon_test = PointInPolygonTest::new(polygons);
*/
        // assign new value to the GRAPH reference
        self.graph = read_or_create_graph("./iceland-coastlines.osm.pbf");
        // self.graph = read_or_create_graph("./planet-coastlines.pbf.sec");
        self.dijkstra = Some(Dijkstra::new(self.graph.adjacency_array(), 3));
        self.nearest_neighbor = Some(NearestNeighbor::new(&self.graph.nodes));
    }

    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute> {
        if let Some(dijkstra) = self.dijkstra.as_mut() {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.start());
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.end());
            dijkstra.change_source_node(start_node);
            if let Some(route_and_distance) = dijkstra.find_route(end_node) {
                let route: Vec<u32> = route_and_distance.0;
                let distance = route_and_distance.1;
                let nodes_route: Vec<Node> = route.into_iter().map(|i| {self.graph.nodes[i as usize]}).collect();
                return Some(ShipRoute::new(nodes_route));
            }
        }
        None
    }

    fn get_number_nodes(&self) -> u32 {
        self.graph.nodes.len() as u32
    }
}
