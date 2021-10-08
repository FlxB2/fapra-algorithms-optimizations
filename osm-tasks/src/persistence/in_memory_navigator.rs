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
use crate::algorithms::bd_dijkstra::BdDijkstra;
use crate::import::pbf_reader::{read_or_create_graph, read_or_create_cn_metadata};
use crate::model::cn_model::CNMetadata;
use crate::algorithms::cn_search::CNBdDijkstra;
use crate::algorithms::unwrap_shortcuts::unwrap_shortcuts;

pub(crate) struct InMemoryGraph {
    graph: GridGraph,
    cn_metadata: CNMetadata,
    dijkstra: Option<Dijkstra>,
    nearest_neighbor: Option<NearestNeighbor>,
}

impl Navigator for InMemoryGraph {
    fn new() -> InMemoryGraph {
        let config = Config::global();

        let cn_metadata = CNMetadata {
            graph: GridGraph::default(),
            get_shortcuts: HashMap::new(),
        };

        if config.build_graph_on_startup() {
            let graph = read_or_create_graph(config.coastlines_file(), config.force_rebuild_graph(), config.number_of_nodes());
            let dijkstra = Some(Dijkstra::new(graph.adjacency_array(), graph.nodes.len() as u32 - 1));
            let nearest_neighbor = Some(NearestNeighbor::new(&graph.nodes));
            let cn_metadata = read_or_create_cn_metadata(config.coastlines_file(), config.force_rebuild_graph(), config.number_of_nodes(), &graph);
            InMemoryGraph {
                graph,
                cn_metadata,
                dijkstra,
                nearest_neighbor,
            }
        } else {
            InMemoryGraph {
                graph: GridGraph::default(),
                dijkstra: None,
                cn_metadata,
                nearest_neighbor: None,
            }

        }
    }

    fn build_graph(&mut self, number_nodes: usize) {
        let config = Config::global();
        self.graph = read_or_create_graph(config.coastlines_file(), config.force_rebuild_graph(), number_nodes);
        self.dijkstra = Some(Dijkstra::new(self.graph.adjacency_array(), (number_nodes - 1) as u32));
        self.nearest_neighbor = Some(NearestNeighbor::new(&self.graph.nodes));

        self.cn_metadata = read_or_create_cn_metadata(config.coastlines_file(), config.force_rebuild_graph(), number_nodes, &self.graph);
    }

    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute> {
        if let Some(dijkstra) = self.dijkstra.as_mut() {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.start());
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&route_request.end());
            let start_time = Instant::now();
            let mut tmp = AStar::new(&self.cn_metadata.graph, start_node);
            //tmp.change_source_node(start_node);
            if let Some(route_and_distance) = tmp.find_route(end_node) {
                let route: Vec<u32> = route_and_distance.0;
                let distance = route_and_distance.1;
                let nodes_route: Vec<Node> = route.into_iter().map(|i| { self.graph.nodes[i as usize] }).collect();
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
            let nodes_route: Vec<Node> = route.into_iter().map(|i| { self.graph.nodes[i as usize] }).collect();
            let time: u128 = start_time.elapsed().as_nanos();
            println!("Dikstra calculated route from {} to {} with distance {} in {} ns, or {} ms", start_node, end_node, distance, start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"),
                query_id,
                amount_nodes_popped: route_and_distance.2,
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
            let nodes_route: Vec<Node> = route.into_iter().map(|i| { self.graph.nodes[i as usize] }).collect();
            let time: u128 = start_time.elapsed().as_nanos();
            println!("A Star calculated route from {} to {} with distance {} and number_nodes {} in {} ns, or {} ms",
                     start_node, end_node, distance, nodes_route.len(), start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"),
                query_id,
                amount_nodes_popped: route_and_distance.2,
            });
        }
        None
    }

    fn benchmark_bd_dijkstra(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult> {
        // completely re initialize dijkstra to make sure nothing is cached or anything similar
        let mut bd_dijkstra = BdDijkstra::new(&self.graph, start_node);
        let start_time = Instant::now();
        if let Some(route_and_distance) = bd_dijkstra.find_route(end_node) {
            let route: Vec<u32> = route_and_distance.0;
            let distance = route_and_distance.1;
            let nodes_route: Vec<Node> = route.into_iter().map(|i| { self.graph.nodes[i as usize] }).collect();
            let time: u128 = start_time.elapsed().as_nanos();
            println!("Bd Dijkstra calculated route from {} to {} with distance {} and number_nodes {} in {} ns, or {} ms",
                     start_node, end_node, distance, nodes_route.len(), start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"),
                query_id,
                amount_nodes_popped: route_and_distance.2,
            });
        }
        None
    }

    fn get_number_nodes(&self) -> u32 {
        self.graph.nodes.len() as u32
    }

        fn benchmark_ch(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult> {
            let mut ch_bd_dijkstra = Dijkstra::new(self.cn_metadata.graph.adjacency_array(), start_node);
        println!("cn graph edges {} normal graph edges {}", self.cn_metadata.graph.edges.concat().len(), self.graph.edges.concat().len());
        let start_time = Instant::now();
        if let Some(route_and_distance) = ch_bd_dijkstra.find_route(end_node) {
            println!("number nodes before {}", route_and_distance.0.len());
            let route: Vec<u32> = unwrap_shortcuts(&route_and_distance.0, &self.cn_metadata.get_shortcuts);
            let distance = route_and_distance.1;
            let nodes_route: Vec<Node> = route.into_iter().map(|i| { self.graph.nodes[i as usize] }).collect();
            let time: u128 = start_time.elapsed().as_nanos();
            println!("CH Dijkstra calculated route from {} to {} with distance {} and number_nodes {} in {} ns, or {} ms",
                     start_node, end_node, distance, nodes_route.len(), start_time.elapsed().as_nanos(), start_time.elapsed().as_millis());
            return Some(BenchmarkResult {
                start_node: self.graph.nodes[start_node as usize],
                end_node: self.graph.nodes[end_node as usize],
                nmb_nodes: nodes_route.len(),
                distance,
                time: u64::try_from(time).expect("time too big"),
                query_id,
                amount_nodes_popped: route_and_distance.2,
            });
        }
        None
    }

    fn run_benchmarks(&mut self, nmb_queries: usize) -> CollectedBenchmarks {
        println!("starting benchmarks");
        let mut dijkstra_results_list: Vec<BenchmarkResult> = vec![];
        let mut dijkstra_time_per_distance: Vec<f32> = vec![];
        let mut a_star_results_list: Vec<BenchmarkResult> = vec![];
        let mut bd_dijkstra_results_list: Vec<BenchmarkResult> = vec![];
        let mut ch_results_list: Vec<BenchmarkResult> = vec![];

        let random_nodes: Vec<Node> = self.graph.nodes.choose_multiple(&mut rand::thread_rng(), nmb_queries + 1).cloned().collect();

        for i in 0..random_nodes.len() - 1 {
            let start_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&random_nodes[i]);
            let end_node = self.nearest_neighbor.as_ref().unwrap().find_nearest_neighbor(&random_nodes[i + 1]);

            // BASELINE DIJKSTRA, every result has to be equivalent
            let possible_dijkstra_result = self.benchmark_dijkstra(start_node, end_node, i);

            if possible_dijkstra_result.is_none() {
                // some routes can not be calculated because there exist some nodes without neighbors
                continue;
            }

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
                    println!("{}BAD RESULT A STAR nmb nodes dijkstra {} nmb nodes a star {} length diff {}{}",
                             color::Fg(color::Red), a_star_res.nmb_nodes, dijkstra_result.nmb_nodes, dijkstra_result.distance as i32 - a_star_res.distance as i32, color::Fg(color::Reset))
                }
            }

            let bd_dijkstra_result = self.benchmark_bd_dijkstra(start_node, end_node, i);
            if let Some(bd_dijkstra_res) = bd_dijkstra_result {
                if bd_dijkstra_res.nmb_nodes == dijkstra_result.nmb_nodes && dijkstra_result.distance == bd_dijkstra_res.distance {
                    bd_dijkstra_results_list.push(bd_dijkstra_res);
                    let time_diff: i64 = dijkstra_result.time as i64 - bd_dijkstra_res.time as i64;
                    println!("Got bd dijkstra result with time {} diff to dijkstra {}", bd_dijkstra_res.time, time_diff);
                } else {
                    println!("{}BAD RESULT BD DIJKSTRA nmb nodes dijkstra {} nmb nodes bd dijkstra {} length diff {}{}",
                             color::Fg(color::Red), dijkstra_result.nmb_nodes, bd_dijkstra_res.nmb_nodes, dijkstra_result.distance as i32 - bd_dijkstra_res.distance as i32, color::Fg(color::Reset))
                }
            }

            let ch_result = self.benchmark_ch(start_node, end_node, i);
            if let Some(ch_res) = ch_result {
                if ch_res.nmb_nodes == dijkstra_result.nmb_nodes && dijkstra_result.distance == ch_res.distance {
                    ch_results_list.push(ch_res);
                    let time_diff: i64 = dijkstra_result.time as i64 - ch_res.time as i64;
                    println!("Got ch result with time {} diff to dijkstra {}", ch_res.time, time_diff);
                } else {
                    println!("{}BAD RESULT CH nmb nodes dijkstra {} nmb nodes ch {} length diff {} time {}{}",
                             color::Fg(color::Red), dijkstra_result.nmb_nodes, ch_res.nmb_nodes, dijkstra_result.distance as i32 - ch_res.distance as i32, ch_res.time, color::Fg(color::Reset))
                }
            }
        }
        let results = CollectedBenchmarks {
            dijkstra: AlgoBenchmark {
                results: dijkstra_results_list,
            },
            a_star: AlgoBenchmark {
                results: a_star_results_list,
            },
            bd_dijkstra: AlgoBenchmark {
                results: bd_dijkstra_results_list,
            },
            ch: AlgoBenchmark {
                results: ch_results_list,
            },
        };

        return results;
    }
}
