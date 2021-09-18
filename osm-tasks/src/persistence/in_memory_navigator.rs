use crate::pbf_reader::{read_or_create_graph};
use crate::persistence::navigator::Navigator;
use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use crate::config::Config;
use rand::seq::{SliceRandom};
use std::time::Instant;
use crate::model::benchmark::{AlgoBenchmark, BenchmarkResult, CollectedBenchmarks};
use std::collections::HashMap;
use crate::algorithms::dijkstra::Dijkstra;
use crate::algorithms::nearest_neighbor::NearestNeighbor;
use crate::model::grid_graph::{GridGraph, Node};
use crate::algorithms::a_star::AStar;
use std::convert::TryFrom;
use termion::color;

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

    fn benchmark_dijkstra(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult> {
        // completely re initialize dijkstra to make sure nothing is cached or anything similar
        let mut dijkstra = Dijkstra::new(self.graph.adjacency_array(), start_node);
        let start_time = Instant::now();
        dijkstra.change_source_node(start_node);
        if let Some(route_and_distance) = dijkstra.find_route(end_node) {
            let route: Vec<u32> = route_and_distance.0;
            let distance = route_and_distance.1;
            let nodes_route: Vec<Node> = route.into_iter().map(|i| {self.graph.nodes[i as usize]}).collect();
            let time: u128 = start_time.elapsed().as_millis();
            println!("Dikstra calculated route from {} to {} with distance {} in {} ns, or {} ms", start_node, end_node, distance, start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"), query_id
            });
        }
        None
    }

    fn benchmark_a_star(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult> {
        // completely re initialize dijkstra to make sure nothing is cached or anything similar
        let mut a_star = AStar::new(&self.graph, start_node);
        let start_time = Instant::now();
        if let Some(route_and_distance) = a_star.find_route(end_node) {
            let route: Vec<u32> = route_and_distance.0;
            let distance = route_and_distance.1;
            let nodes_route: Vec<Node> = route.into_iter().map(|i| {self.graph.nodes[i as usize]}).collect();
            let time: u128 = start_time.elapsed().as_millis();
            println!("AStar calculated route from {} to {} with distance {} in {} ns, or {} ms", start_node, end_node, distance, start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"), query_id
            });
        }
        None
    }

    fn get_number_nodes(&self) -> u32 {
        self.graph.nodes.len() as u32
    }

    fn run_benchmarks(&mut self, nmb_queries: usize) -> CollectedBenchmarks {
        println!("starting benchmarks");
        let mut dijkstra_results_list : Vec<BenchmarkResult> = vec![];
        let mut dijkstra_time_per_distance: Vec<f32> = vec![];
        let mut a_star_results_list : Vec<BenchmarkResult> = vec![];
        let mut a_star_time_per_distance: Vec<f32> = vec![];
        let random_nodes: Vec<Node> = self.graph.nodes.choose_multiple(&mut rand::thread_rng(), nmb_queries+1).cloned().collect();

        for i in 0..random_nodes.len()-1 {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&random_nodes[i]);
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&random_nodes[i+1]);
            let distance = self.graph.get_distance(start_node,end_node) as u64;

            // BASELINE DIJKSTRA, every result has to be equivalent
            let possible_dijkstra_result = self.benchmark_dijkstra(start_node, end_node, i);
            let dijkstra_result = possible_dijkstra_result.expect("dijkstra result not available");
            dijkstra_time_per_distance.push((dijkstra_result.distance as u64 / dijkstra_result.time) as f32);
            dijkstra_results_list.push(dijkstra_result);
            println!("Got dijkstra result with time {}", dijkstra_result.time);


            let a_star_result = self.benchmark_a_star(start_node, end_node, i);
            if let Some(a_star_res) = a_star_result {
                if a_star_res.nmb_nodes == dijkstra_result.nmb_nodes && dijkstra_result.distance == a_star_res.distance {
                    a_star_results_list.push(a_star_res);
                    let time_diff: i64 = dijkstra_result.time as i64 - a_star_res.time as i64;
                    println!("Got a_star result with time {} diff to dijkstra {}", a_star_res.time, time_diff);
                } else {
                    println!("{}BAD RESULT A STAR nmb nodes dijkstra {} nmb nodes a star {} length diff {}",
                             color::Fg(color::Red), a_star_res.nmb_nodes, dijkstra_result.nmb_nodes, dijkstra_result.distance-a_star_res.distance)
                }
                a_star_time_per_distance.push((a_star_res.distance as u64 / a_star_res.time) as f32);
            }
        }
        let mut results: HashMap<String, AlgoBenchmark> = HashMap::new();
        results.insert(String::from("Dijkstra"),  AlgoBenchmark {
            results: dijkstra_results_list,
            avg_distance_per_ms: average(dijkstra_time_per_distance.as_slice())
        });
        results.insert(String::from("AStar"),  AlgoBenchmark {
            results: a_star_results_list,
            avg_distance_per_ms: average(a_star_time_per_distance.as_slice())
        });


        CollectedBenchmarks {
            results
        }
    }
}

fn average(numbers: &[f32]) -> f32 {
    numbers.iter().sum::<f32>() as f32 / numbers.len() as f32
}
