use std::collections::{HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::{GridGraph, Edge};
use crate::model::heap_item::HeapItem;
use crate::algorithms::witness_search::WitnessSearch;
use rand::Rng;
use rand::distributions::Uniform;
use crate::model::cn_model::{Shortcut, CNMetadata};
use std::time::Instant;

pub(crate) struct CNGraphCreator<'a> {
    graph_ref: &'a GridGraph,
    modified_graph: GridGraph,
    contracted_nodes: HashMap<u32, bool>,
    // format to create unique key s.edge.source.to_string() + &*s.edge.target.to_string()
    // value is index of shortcut in self.shortcut - enables faster unwrapping
    get_shortcut: HashMap<String, Shortcut>,
}

impl<'a> CNGraphCreator<'a> {
    pub fn new(graph: &GridGraph) -> CNGraphCreator {
        let number_of_nodes = graph.nodes.len() as usize;

        return CNGraphCreator {
            graph_ref: graph,
            modified_graph: GridGraph {
                number_edges: 0,
                number_nodes: 0,
                edges: vec![],
                nodes: vec![],
            },
            contracted_nodes: HashMap::new(),
            get_shortcut: HashMap::new(),
        };
    }

    pub fn build_cn_graph(&mut self) -> CNMetadata {
        println!("starting to create cn metadata");
        let number_edges_before = self.graph_ref.edges.concat().len();
        let mut collected_nodes = 0.0;
        self.modified_graph = (*self.graph_ref).clone();
        let mut removed_nodes: HashMap<u32, bool> = HashMap::new();
        let i_set_size = ((1.0 / 100.0) * self.modified_graph.number_nodes as f64) as usize;

        let start_time = Instant::now();

        // we aim to contract 90% of nodes
        while collected_nodes < ((9.0 / 10.0) * self.graph_ref.number_nodes as f64) {
            println!("generating new independent set {} ms", start_time.elapsed().as_millis());
            let modified_adj_array = self.modified_graph.adjacency_array();

            // create independent set by picking nodes randomly
            let mut disallowed_nodes: HashMap<u32, bool> = HashMap::new();
            let mut has_enough_nodes = false;
            let mut independent_set = Vec::with_capacity(i_set_size as usize);
            let range = Uniform::from(0..(self.modified_graph.nodes.len() as u32));
            let sample_size = i_set_size * 2;

            while !has_enough_nodes {
                let choices: Vec<u32> = rand::thread_rng().sample_iter(&range).take(sample_size as usize).collect();
                //println!("generated random numbers {} ms", start_time.elapsed().as_millis());
                let mut cnt: i32 = -1;
                'node: for node in choices {
                    cnt += 1;
                    if removed_nodes.contains_key(&node) {
                        if cnt == sample_size as i32 {
                            // we looked at all samples, sample again random values
                            break 'node;
                        }
                        continue 'node;
                    }
                    let neighbors_distances = modified_adj_array.get_neighbors_of_node_and_distances(node);
                    for neighbor in (0..neighbors_distances.len()).step_by(2) {
                        if disallowed_nodes.contains_key(&(neighbor as u32)) {
                            continue 'node;
                        }
                    }
                    // found an independent node!
                    independent_set.push(node);
                    disallowed_nodes.insert(node, true);
                    //println!("set size {}, i_set_size {}", independent_set.len(), i_set_size);

                    // stopping condition, we reached i_set_size nodes
                    if independent_set.len() == i_set_size as usize {
                        has_enough_nodes = true;
                        break 'node;
                    }
                }
            }
            println!("found independent set w/ size {} in {} ms", i_set_size, start_time.elapsed().as_millis());

            let mut rank_map: Vec<Vec<u32>> = vec![vec![]; 15];

            // create map of ranks and remove nodes from graph
            for i in 0..independent_set.len() {
                collected_nodes += 1.0;

                // contraction order heuristic: out-degree
                let curr_rank = calc_number_edges(independent_set[i], &self.graph_ref) as usize;
                rank_map[curr_rank].push(independent_set[i]);
            }
            println!("collected nodes {} in {} ms", collected_nodes, start_time.elapsed().as_millis());

            let mut index_max_nmb_nodes_in_rank= 0;
            let mut max_nmb_nodes = 0;
            for rank in 0..15 {
                if rank_map[rank].len() > max_nmb_nodes {
                    max_nmb_nodes = rank_map[rank].len();
                    index_max_nmb_nodes_in_rank = rank;
                }
                println!("rank {}, len {}", rank, rank_map[rank].len());
            }
            // we don't create shortcuts for the core
            rank_map[index_max_nmb_nodes_in_rank] = vec![];

            // we only contract nodes with a low rank, most nodes are in rank 8, which we
            // skip to leave an un-contracted core
            for i in 1..7 {
                // we only create shortcuts between lower to higher ranks
                let destinations: &[u32] = &rank_map[i + 1..rank_map.len()].concat();
                for j in 0..rank_map[i].len() {
                    // find route from node i,j to destinations
                    self.find_shortcuts(rank_map[i][j], destinations, &modified_adj_array, &removed_nodes);
                }
                println!("rank {} done in {} ms", i, start_time.elapsed().as_millis());
            }
            println!("found shortcuts between nodes, nmb already collected: {}, time: {} ms", self.get_shortcut.keys().len(), start_time.elapsed().as_millis());

            // remove nodes for sweeps after this one
            for i in 0..independent_set.len() {
                self.modified_graph.remove_node(independent_set[i]);
                removed_nodes.insert(independent_set[i], true);
            }
            println!("removed nodes from graph in {}", start_time.elapsed().as_millis());
        }

        // final graph with added shortcuts
        println!("found {} shortcuts, after {} ms", self.get_shortcut.keys().len(), start_time.elapsed().as_millis());
        let mut final_graph = (*self.graph_ref).clone();
        for s in self.get_shortcut.values() {
            // add shortcuts to initial graph
            final_graph.add_new_edge(s.edge);
        }
        println!("edges before {}, after {}", number_edges_before, final_graph.edges.concat().len());
        println!("added {} shortcuts", self.get_shortcut.keys().len());

        println!("finished building cn metadata - started copying graph after {} ms", start_time.elapsed().as_millis());
        return CNMetadata {
            graph: final_graph,
            get_shortcut: self.get_shortcut.clone()
        };
    }

    fn find_shortcuts(&mut self, node: u32, dest: &[u32], adj_array: &AdjacencyArray, removed_nodes: &HashMap<u32, bool>) {
        let mut dijkstra = WitnessSearch::new(adj_array, node, removed_nodes);
        self.contracted_nodes.insert(node, true);

        // TODO actually calculate result between neighbors, not other nodes from the independent set

        dijkstra.change_source_node(node);
        // TODO check if uvw = length of route found (v contracted, u,w neighbors) [STALL ON DEMAND]
        // TODO this ensures no suboptimal shortcuts are added (and that u is always included)
        let dist_uvw = 0;
        if let Some(result) = dijkstra.find_route(dest) {
            let routes = result.0;
            let distances = result.1;
            let mut counter = 0;
            for route in routes {
                let edge = Edge {
                    source: route[0],
                    target: route[route.len() - 1],
                    distance: distances[counter],
                };

                // ignore duplicates - key is unique id which will not change
                let key = edge.source.to_string() + "_" + &*edge.target.to_string();
                if !self.get_shortcut.contains_key(&*key) {
                    // TODO consider adding shortcut in both directions
                    // save shortcut for quicker unwrapping later on + fast query if edge is a shortcut
                    self.get_shortcut.insert(key, Shortcut {
                        replaced_edges: route,
                        edge,
                    });
                    self.modified_graph.add_new_edge(edge);
                }

                counter += 1;
            }
        }
    }
}

fn calc_number_edges(v: u32, graph: &GridGraph) -> u32 {
    return graph.edges[v as usize].len() as u32;
}
