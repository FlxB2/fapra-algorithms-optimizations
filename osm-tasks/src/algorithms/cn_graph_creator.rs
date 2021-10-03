use std::collections::{BinaryHeap, HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::{GridGraph, Edge, Node};
use crate::model::heap_item::HeapItem;
use crate::algorithms::dijkstra::Dijkstra;
use crate::algorithms::witness_search::WitnessSearch;
use rand_xorshift::XorShiftRng;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;
use rand::Rng;
use rand::distributions::Uniform;
use rand::prelude::SliceRandom;
use crate::model::cn_model::{Shortcut, CNMetadata};

pub(crate) struct CNGraphCreator<'a> {
    graph_ref: &'a GridGraph,
    modified_graph: GridGraph,
    forward_heap: BinaryHeap<HeapItem>,
    backward_heap: BinaryHeap<HeapItem>,
    forward_distances: Vec<u32>,
    backward_distances: Vec<u32>,
    forward_previous_nodes: Vec<u32>,
    backward_previous_nodes: Vec<u32>,
    amount_nodes_popped_forward: usize,
    amount_nodes_popped_backward: usize,
    mu: u32,
    meeting_node: u32,
    shortcuts: Vec<Shortcut>,
    contracted_nodes: HashMap<u32, bool>,
    // format to create unique key s.edge.source.to_string() + &*s.edge.target.to_string()
    is_shortcut: HashMap<String, bool>,
    removed_edges: HashMap<u32, bool>,
}

impl<'a> CNGraphCreator<'a> {
    pub fn new(graph: &GridGraph) -> CNGraphCreator {
        let number_of_nodes = graph.nodes.len() as usize;
        let forward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let backward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let forward_distances = vec![u32::MAX; number_of_nodes];
        let backward_distances = vec![u32::MAX; number_of_nodes];
        let forward_previous_nodes = vec![u32::MAX; number_of_nodes];
        let backward_previous_nodes = vec![u32::MAX; number_of_nodes];

        return CNGraphCreator {
            graph_ref: graph,
            modified_graph: GridGraph {
                number_edges: 0,
                number_nodes: 0,
                edges: vec![],
                nodes: vec![],
            },
            forward_heap,
            backward_heap,
            forward_distances,
            backward_distances,
            forward_previous_nodes,
            backward_previous_nodes,
            amount_nodes_popped_forward: 0,
            amount_nodes_popped_backward: 0,
            mu: u32::MAX,
            meeting_node: u32::MAX,
            shortcuts: vec![],
            contracted_nodes: HashMap::new(),
            is_shortcut: HashMap::new(),
            removed_edges: HashMap::new(),
        };
    }

    pub fn build_cn_graph(&mut self) -> CNMetadata {
        println!("starting to create cn metadata");
        let number_edges_before = self.graph_ref.edges.concat().len();
        let mut collected_nodes = 0.0;
        self.modified_graph = (self.graph_ref).clone();
        let mut removed_nodes: HashMap<u32, bool> = HashMap::new();
        let mut i_set_size = ((1.0 / 100.0) * self.modified_graph.number_nodes as f64) as usize;

        // we aim to contract 90% of nodes
        while collected_nodes < ((9.0 / 10.0) * self.graph_ref.number_nodes as f64) {
            let mut modified_adj_array = self.modified_graph.adjacency_array();

            // create independent set by picking nodes randomly
            let mut disallowed_nodes: HashMap<u32, bool> = HashMap::new();
            let mut has_enough_nodes = false;
            let mut independent_set = Vec::with_capacity(i_set_size as usize);
            let range = Uniform::from(0..(self.modified_graph.nodes.len() as u32));

            while !has_enough_nodes {
                let choices: Vec<u32> = rand::thread_rng().sample_iter(&range).take(2 * i_set_size as usize).collect();
                let mut cnt: i32 = -1;
                'node: for node in choices {
                    cnt += 1;
                    if removed_nodes.contains_key(&node) {
                        if cnt == i_set_size as i32 {
                            break 'node;
                        }
                        continue 'node;
                    }
                    let neighbors_distances = modified_adj_array.get_neighbors_of_node_and_distances(node);
                    'neighbor: for neighbor in (0..neighbors_distances.len()).step_by(2) {
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
            println!("found independent set w/ size {}", i_set_size);

            let mut rank_map: Vec<Vec<u32>> = vec![vec![]; 15];

            // create map of ranks and remove nodes from graph
            for i in 0..independent_set.len() {
                collected_nodes += 1.0;

                // contraction order heuristic: out-degree
                let curr_rank = calc_number_edges(independent_set[i], &self.graph_ref) as usize;
                rank_map[curr_rank].push(independent_set[i]);
            }
            println!("collected nodes {}", collected_nodes);

            // TODO this might be shit, way too many duplicate paths? okay maybe not
            for i in 0..rank_map.len() - 1 {
                // we only create shortcuts between lower to higher ranks
                let destinations: &[u32] = &rank_map[i + 1..rank_map.len()].concat();
                for j in 0..rank_map[i].len() {
                    // find route from node i,j to destinations
                    self.find_shortcuts(rank_map[i][j], destinations, &modified_adj_array, &removed_nodes);
                }
            }

            // remove nodes for sweeps after this one
            for i in 0..independent_set.len() {
                self.modified_graph.remove_node(independent_set[i]);
                removed_nodes.insert(independent_set[i], true);
            }
        }

        // final graph with added shortcuts
        println!("found {} shortcuts", self.shortcuts.len());
        let mut final_graph = (*self.graph_ref).clone();
        for s in &self.shortcuts {
            // add shortcuts to initial graph
            final_graph.add_new_edge(s.edge);
        }
        println!("edges before {}, after {}", number_edges_before, self.graph_ref.edges.concat().len());
        println!("added {} shortcuts", self.shortcuts.len());

        println!("finished building cn metadata - started copying graph");
        return CNMetadata {
            graph: final_graph,
            shortcuts: self.shortcuts.clone(),
            is_shortcut: self.is_shortcut.clone()
        };
    }

    fn find_shortcuts(&mut self, node: u32, dest: &[u32], adj_array: &AdjacencyArray, removed_nodes: &HashMap<u32, bool>) {
        let mut dijkstra = WitnessSearch::new(adj_array, node, removed_nodes);
        self.contracted_nodes.insert(node, true);

        dijkstra.change_source_node(node);
        // TODO check if uvw = length of route found (v contracted, u,w neighbors)
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
                let key = edge.source.to_string() + &*edge.target.to_string();
                if !self.is_shortcut.contains_key(&*key) {
                    self.is_shortcut.insert(key, true);
                    self.modified_graph.add_new_edge(edge);

                    // TODO consider addding shortcut in both directions
                    self.shortcuts.push(Shortcut {
                        replaced_edges: route,
                        edge,
                    });
                }

                counter += 1;
            }
        }
    }

    /*
    pub fn find_route(&mut self, destination_node: u32) -> Option<(Vec<u32>, u32, u32)> {
        let meeting_node = self.bd_dijkstra(self.source_node, destination_node);

        let mut route = vec![];
        let mut current = meeting_node;
        while current != self.source_node {
            route.push(current);
            current = self.forward_previous_nodes[current as usize];
        }
        route.push(self.source_node);
        route.reverse();
        current = self.backward_previous_nodes[meeting_node as usize];
        while current != destination_node {
            route.push(current);
            current = self.backward_previous_nodes[current as usize];
        }
        route.push(destination_node);

        Some((route,
              self.forward_distances[meeting_node as usize] + self.backward_distances[meeting_node as usize],
              (self.amount_nodes_popped_forward + self.amount_nodes_popped_backward) as u32))
    }



    fn bd_dijkstra(&mut self, source_node: u32, destination_node: u32) -> u32 {
        let adj_array = self.graph_ref.adjacency_array();
        self.meeting_node = u32::MAX;
        self.mu = u32::MAX;

        self.backward_heap.push(HeapItem {
            node_id: destination_node,
            distance: 0,
            previous_node: destination_node,
        });
        self.forward_heap.push(HeapItem {
            node_id: source_node,
            distance: 0,
            previous_node: source_node,
        });
        self.forward_distances[source_node as usize] = 0;
        self.backward_distances[destination_node as usize] = 0;

        loop {
            let curr_mu = self.forward_heap.peek().unwrap().distance + self.backward_heap.peek().unwrap().distance;

            if curr_mu >= self.mu {
                return self.meeting_node;
            }

            if self.forward_heap.len() + self.amount_nodes_popped_forward < self.backward_heap.len() + self.amount_nodes_popped_backward {
                self.expand_forward(&adj_array);
            } else {
                self.expand_backward(&adj_array);
            }
        }
    }

    fn expand_forward(&mut self, adj_array: &AdjacencyArray) {
        let current = self.forward_heap.pop();
        if let Some(curr) = current {
            let neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(curr.node_id);

            // iterate over children
            for i in (0..neighbors_and_distances.len()).step_by(2) {
                let neighbor = neighbors_and_distances[i];
                let neighbor_distance = neighbors_and_distances[i + 1];

                let score = curr.distance + neighbor_distance;

                if self.forward_distances[neighbor as usize] == u32::MAX || self.forward_distances[neighbor as usize] > score {
                    // we did not encounter this node before
                    self.forward_previous_nodes[neighbor as usize] = curr.node_id;
                    self.forward_distances[neighbor as usize] = score;
                    self.forward_heap.push(HeapItem {
                        distance: score,
                        node_id: neighbor,
                        previous_node: curr.node_id,
                    });
                    self.update_best_path_forward(neighbor as usize, score);
                }
            }
        }
    }

    fn expand_backward(&mut self, adj_array: &AdjacencyArray) {
        let current = self.backward_heap.pop();
        if let Some(curr) = current {
            let neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(curr.node_id);

            // iterate over children
            for i in (0..neighbors_and_distances.len()).step_by(2) {
                let neighbor = neighbors_and_distances[i];
                let neighbor_distance = neighbors_and_distances[i + 1];

                let score = curr.distance + neighbor_distance;

                if self.backward_distances[neighbor as usize] == u32::MAX || self.backward_distances[neighbor as usize] > score {
                    // we did not encounter this node before
                    self.backward_previous_nodes[neighbor as usize] = curr.node_id;
                    self.backward_distances[neighbor as usize] = score;
                    self.backward_heap.push(HeapItem {
                        distance: score,
                        node_id: neighbor,
                        previous_node: curr.node_id,
                    });
                    self.update_best_path_backward(neighbor as usize, score);
                }
            }
        }
    }

    fn update_best_path_forward(&mut self, neighbor: usize, score: u32) -> bool {
        if self.backward_previous_nodes[neighbor as usize] != u32::MAX {
            // backward search already found this node
            let new_mu = self.backward_distances[neighbor as usize] + score;
            if self.mu > new_mu {
                self.mu = new_mu;
                self.meeting_node = neighbor as u32;
                return true;
            }
        }
        false
    }

    fn update_best_path_backward(&mut self, neighbor: usize, score: u32) -> bool {
        if self.forward_previous_nodes[neighbor as usize] != u32::MAX {
            // backward search already found this node
            let new_mu = self.forward_distances[neighbor as usize] + score;
            if self.mu > new_mu {
                self.mu = new_mu;
                self.meeting_node = neighbor as u32;
                return true;
            }
        }
        false
    } */
}

fn calc_number_edges(v: u32, graph: &GridGraph) -> u32 {
    return graph.edges[v as usize].len() as u32;
}
