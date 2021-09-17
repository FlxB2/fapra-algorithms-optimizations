use crate::grid_graph::{GridGraph, Node};
use crate::pbf_reader::{read_or_create_graph};
use crate::persistence::navigator::Navigator;
use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use crate::config::Config;
use rand::seq::{SliceRandom, IteratorRandom};
use std::time::Instant;
use crate::benchmark::{AlgoBenchmark, BenchmarkResult, CollectedBenchmarks};
use std::iter::Map;
use std::collections::HashMap;
use rocket::http::ext::IntoCollection;
use crate::algorithms::dijkstra::Dijkstra;
use crate::algorithms::nearest_neighbor::NearestNeighbor;

pub(crate) struct InMemoryGraph {
    graph: GridGraph,
    dijkstra: Option<Dijkstra>,
    nearest_neighbor: Option<NearestNeighbor>
}

impl Navigator for InMemoryGraph {
    fn new() -> InMemoryGraph {
        let config = Config::global();
        if config.build_graph_on_startup() {
            let graph =  read_or_create_graph(config.coastlines_file(), false);
            let dijkstra = Some(Dijkstra::new(graph.adjacency_array(), graph.nodes.len() as u32 - 1));
            let nearest_neighbor = Some(NearestNeighbor::new(&graph.nodes));
            InMemoryGraph {
                graph,
                dijkstra,
                nearest_neighbor
            }
        } else {
            InMemoryGraph {
                graph: GridGraph::default(),
                dijkstra: None,
                nearest_neighbor: None
            }
        }
    }

    fn build_graph(&mut self) {
        /*let polygons =
        //let polygons = read_file("./iceland-coastlines.osm.pbf");
        let polygon_test = PointInPolygonTest::new(polygons);
*/
        // assign new value to the GRAPH reference
        // self.graph = read_or_create_graph("./iceland-coastlines.osm.pbf");
        // self.graph = read_or_create_graph("./planet-coastlines.pbf.sec");
        let config = Config::global();
        self.graph = read_or_create_graph(config.coastlines_file(), config.force_rebuild_graph());
        self.dijkstra = Some(Dijkstra::new(self.graph.adjacency_array(), self.get_number_nodes() - 1));
        self.nearest_neighbor = Some(NearestNeighbor::new(&self.graph.nodes));
    }

    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute> {
        if let Some(dijkstra) = self.dijkstra.as_mut() {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.start());
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.end());
            let start_time = Instant::now();
            dijkstra.change_source_node(start_node);
            if let Some(route_and_distance) = dijkstra.find_route(end_node) {
                let route: Vec<u32> = route_and_distance.0;
                let distance = route_and_distance.1;
                let nodes_route: Vec<Node> = route.into_iter().map(|i| {self.graph.nodes[i as usize]}).collect();
                println!("Calculated route from {} to {} with distance {} in {} ns, or {} ms", start_node, end_node, distance, start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
                return Some(ShipRoute::new(nodes_route, distance));
            } else {
                println!("Could not calculate route. Dijkstra is not initialized");
            }
        }
        None
    }

    fn benchmark_dijkstra(&mut self, route_request: RouteRequest, query_id: usize) -> Option<BenchmarkResult> {
        // completely re initialize dijkstra to make sure nothing is cached or anything similar
        self.dijkstra = Some(Dijkstra::new(self.graph.adjacency_array(), self.get_number_nodes() - 1));
        if let Some(dijkstra) = self.dijkstra.as_mut() {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.start());
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.end());
            let start_time = Instant::now();
            dijkstra.change_source_node(start_node);
            if let Some(route_and_distance) = dijkstra.find_route(end_node) {
                let route: Vec<u32> = route_and_distance.0;
                let distance = route_and_distance.1;
                let nodes_route: Vec<Node> = route.into_iter().map(|i| {self.graph.nodes[i as usize]}).collect();
                let time: u128 = start_time.elapsed().as_nanos();
                println!("Calculated route from {} to {} with distance {} in {} ns, or {} ms", start_node, end_node, distance, start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
                return Some(BenchmarkResult {
                    start_node: route_request.start,
                    end_node: route_request.end,
                    nmb_nodes: nodes_route.len(),
                    time, query_id
                });
            } else {
                println!("Could not calculate route. Dijkstra is not initialized");
            }
        }
        None
    }

    fn get_number_nodes(&self) -> u32 {
        self.graph.nodes.len() as u32
    }

    fn run_benchmarks(&mut self, nmb_queries: usize) -> CollectedBenchmarks {
        println!("starting benchmarks");
        let mut dijkstra_results_list : Vec<BenchmarkResult> = vec![];
        let random_nodes: Vec<Node> = self.graph.nodes.choose_multiple(&mut rand::thread_rng(), nmb_queries).cloned().collect();

        for i in 0..random_nodes.len()-1 {
            let route_request = RouteRequest {
                start: random_nodes[i].clone(),
                end: random_nodes[i + 1].clone()
            };

            // BASELINE DIJKSTRA, every result has to be equivalent
            let dijkstra_result = self.benchmark_dijkstra(route_request, i);
            if let Some(dijkstra_res) = dijkstra_result {
                dijkstra_results_list.push(dijkstra_res);
                println!("Got dijkstra result with time {}", dijkstra_res.time);
            }

            // TODO: are routes the same? [probably just check length, its enough regarding resolution of 1mio nodes]

        }
        let mut results: HashMap<String, AlgoBenchmark> = HashMap::new();
        results.insert(String::from("Dijkstra"),  AlgoBenchmark {
            results: dijkstra_results_list,
            avg_time_per_distance: 0
        });


        CollectedBenchmarks {
            results
        }
    }
}
