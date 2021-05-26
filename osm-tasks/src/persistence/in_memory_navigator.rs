use crate::grid_graph::GridGraph;
use crate::pbf_reader::read_file;
use crate::polygon_test::PointInPolygonTest;
use crate::persistence::navigator::Navigator;
use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use std::sync::Mutex;

pub(crate) struct InMemoryGraph {
    graph: GridGraph
}

impl Navigator for InMemoryGraph {
    fn new() -> InMemoryGraph {
        InMemoryGraph {
            graph: GridGraph::default()
        }
    }

    fn build_graph(&mut self) {
        //let polygons = read_file("./planet-coastlines.pbf.sec");
        let polygons = read_file("./iceland-coastlines.osm.pbf");
        let polygon_test = PointInPolygonTest::new(polygons);

        // assign new value to the GRAPH reference
        self.graph = GridGraph::new(polygon_test);
    }

    fn calculate_route(&self, route_request: RouteRequest) -> Option<ShipRoute> {
        // do dijkstra here
        None
    }

    fn get_number_nodes(&self) -> u32 {
        self.graph.nodes.len() as u32
    }
}
