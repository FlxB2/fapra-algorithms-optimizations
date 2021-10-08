use std::collections::{HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::{GridGraph, Edge};
use crate::model::heap_item::HeapItem;
use crate::algorithms::witness_search::WitnessSearch;
use rand::Rng;
use rand::distributions::Uniform;
use crate::model::cn_model::{Shortcut, CNMetadata};
use std::time::Instant;
use std::collections::hash_map::Entry;

pub(crate) struct CNGraphCreator<'a> {
    graph_ref: &'a GridGraph,
    modified_graph: GridGraph,
    contracted_nodes: HashMap<u32, bool>,
    // format to create unique key s.edge.source.to_string() + &*s.edge.target.to_string()
    // value is index of shortcut in self.shortcut - enables faster unwrapping
    get_shortcuts: HashMap<u32, Vec<Shortcut>>,
    number_shortcuts_added: u64,
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
            get_shortcuts: HashMap::new(),
            number_shortcuts_added: 0
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

            let mut index_max_nmb_nodes_in_rank = 0;
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

            for i in 1..rank_map.len() {
                // we only create shortcuts between lower to higher ranks
                //let destinations: &[u32] = &rank_map[i + 1..rank_map.len()].concat();
                for j in 0..rank_map[i].len() {
                    // find route from node i,j to destinations
                    self.find_shortcuts(rank_map[i][j], &modified_adj_array, &removed_nodes);
                }
                println!("rank {} done in {} ms", i, start_time.elapsed().as_millis());
            }
            println!("found shortcuts between nodes, nmb already collected: {}, time: {} ms", self.number_shortcuts_added, start_time.elapsed().as_millis());

            // remove nodes for sweeps after this one
            for i in 0..independent_set.len() {
                self.modified_graph.remove_node(independent_set[i]);
                removed_nodes.insert(independent_set[i], true);
            }
            println!("removed nodes from graph in {}", start_time.elapsed().as_millis());
        }

        // final graph with added shortcuts
        println!("found {} shortcuts, after {} ms", self.number_shortcuts_added, start_time.elapsed().as_millis());
        let mut final_graph = (*self.graph_ref).clone();
        for e in self.get_shortcuts.values() {
            // add shortcuts to initial graph
            for s in e {
                final_graph.add_new_edge(s.edge);
            }
        }
        println!("edges before {}, after {}", number_edges_before, final_graph.edges.concat().len());
        println!("added {} shortcuts", self.get_shortcuts.keys().len());

        println!("finished building cn metadata - started copying graph after {} ms", start_time.elapsed().as_millis());
        return CNMetadata {
            graph: final_graph,
            get_shortcuts: self.get_shortcuts.clone(),
        };
    }

    fn find_shortcuts(&mut self, node: u32, adj_array: &AdjacencyArray, removed_nodes: &HashMap<u32, bool>) {
        let mut witness_search = WitnessSearch::new(adj_array, node, removed_nodes);
        self.contracted_nodes.insert(node, true);

        let neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(node);
        let mut distances_uv: HashMap<u32, u32> = HashMap::new();

        // calculate sources and destinations of all neighbors of node
        let mut neighbors = vec![];
        for i in (0..neighbors_and_distances.len()).step_by(2) {
            // ignore neighbors which have already been contracted
            if !self.contracted_nodes.contains_key(&neighbors_and_distances[i]) && neighbors_and_distances[i] != u32::MAX {
                neighbors.push(neighbors_and_distances[i]);
                distances_uv.insert(neighbors_and_distances[i], neighbors_and_distances[i+1]);
            }
        }
        if neighbors.len() <= 1 {
            return;
        }

        for j in 0..neighbors.len() {
            let targets: Vec<u32> = neighbors.iter().filter(|&&n| n != neighbors[j]).cloned().collect::<Vec<u32>>();
            witness_search.change_source_node(neighbors[j]);
            if let Some(result) = witness_search.find_route(&*targets) {
                let routes = result.0;
                let distances = result.1;
                let mut counter = 0;
                for route in routes {
                    let distance_uvw = distances_uv[&route[0]] + distances_uv[&route[route.len()-1]];

                    // skip routes which do not contain contracted node or are suboptimal
                    if !route.contains(&node) || distance_uvw <= distances[counter] {
                        continue;
                    }

                    let edge = Edge {
                        source: route[0],
                        target: route[route.len() - 1],
                        distance: distances[counter],
                    };

                    // add shortcut to graph
                    self.modified_graph.add_new_edge(edge);

                    match self.get_shortcuts.entry(route[0]) {
                        Entry::Vacant(e) => {
                            e.insert(vec![Shortcut {
                                replaced_edges: route,
                                edge,
                            }]);
                            self.number_shortcuts_added += 1;
                        }
                        Entry::Occupied(mut e) => {
                            let mut contained = false;
                            for s in e.get() {
                                if s.edge.source == route[0] && s.edge.target == route[route.len() - 1] {
                                    contained = true;
                                }
                            }
                            // ignore duplicates
                            if !contained {
                                e.get_mut().push(Shortcut {
                                    replaced_edges: route,
                                    edge,
                                });
                                self.number_shortcuts_added += 1;
                            }
                        }
                    };

                    counter += 1;
                }
            }
        }
    }
}

fn calc_number_edges(v: u32, graph: &GridGraph) -> u32 {
    return graph.edges[v as usize].len() as u32;
}
