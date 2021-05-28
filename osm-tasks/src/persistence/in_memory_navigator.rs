use crate::grid_graph::{GridGraph, Node};
use crate::pbf_reader::{read_file, read_or_create_graph};
use crate::polygon_test::PointInPolygonTest;
use crate::persistence::navigator::Navigator;
use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use std::sync::Mutex;
use crate::dijkstra::{Dijkstra};

pub(crate) struct InMemoryGraph {
    graph: GridGraph,
    dijkstra: Option<Dijkstra>
}

impl Navigator for InMemoryGraph {
    fn new() -> InMemoryGraph {
        InMemoryGraph {
            graph: GridGraph::default(),
            dijkstra: None
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
        self.dijkstra = Some(Dijkstra::new(self.graph.adjacency_matrix(), 3));
    }

    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute> {
        // Todo: Lookup nearest nodes for source and destination points
        let source = 3;
        let destination = self.graph.nodes.len() as u32 - 1;
        if let Some(dijkstra) = self.dijkstra.as_mut() {
            dijkstra.change_source_node(source);
            if let Some(route_and_distance) = dijkstra.find_route(destination) {
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
