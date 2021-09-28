use std::collections::{BinaryHeap, HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::{GridGraph, Edge};
use crate::model::heap_item::HeapItem;
use crate::algorithms::dijkstra::Dijkstra;
use crate::algorithms::dijkstra_one_to_many::DijkstraToMany;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;

struct Shortcut {
    replaced_edges: Vec<u32>,
    edge: Edge
}

pub(crate) struct ContractionHierarchies {
    graph_ref: GridGraph,
    modified_graph: GridGraph,
    forward_heap: BinaryHeap<HeapItem>,
    backward_heap: BinaryHeap<HeapItem>,
    forward_distances: Vec<u32>,
    backward_distances: Vec<u32>,
    forward_previous_nodes: Vec<u32>,
    backward_previous_nodes: Vec<u32>,
    source_node: u32,
    amount_nodes_popped_forward: usize,
    amount_nodes_popped_backward: usize,
    mu: u32,
    meeting_node: u32,
    shortcuts: Vec<Shortcut>,
    contracted_nodes: HashMap<u32, bool>,
    shortcuts_start_indexes: HashMap<usize, usize>,
    removed_edges: HashMap<u32, bool>
}

impl ContractionHierarchies  {
    pub fn new(graph: GridGraph, source_node: u32) -> ContractionHierarchies {
        let number_of_nodes = graph.nodes.len() as usize;
        // Todo: Ist es sinnvoll den heap mit der Anzahl der Knoten zu initialisieren?
        let forward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let backward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let forward_distances = vec![u32::MAX; number_of_nodes];
        let backward_distances = vec![u32::MAX; number_of_nodes];
        let forward_previous_nodes = vec![u32::MAX; number_of_nodes];
        let backward_previous_nodes = vec![u32::MAX; number_of_nodes];

        return ContractionHierarchies {
            graph_ref: graph,
            modified_graph: GridGraph {
                number_nodes: 0,
                offsets: vec![],
                edges: vec![],
                nodes: vec![],
                //removed_edges: Default::default()
            },
            forward_heap,
            backward_heap,
            forward_distances,
            backward_distances,
            forward_previous_nodes,
            backward_previous_nodes,
            source_node,
            amount_nodes_popped_forward: 0,
            amount_nodes_popped_backward: 0,
            mu: u32::MAX,
            meeting_node: u32::MAX,
            shortcuts: vec![],
            contracted_nodes: HashMap::new(),
            shortcuts_start_indexes: HashMap::new(),
            removed_edges: HashMap::new()
        };
    }

    pub fn preprocessing(&mut self) {
        let mut rank = 1;
        self.modified_graph = (self.graph_ref).clone();
        let mut rank_map: Vec<Vec<u32>> = vec![vec![]; 15];
        println!("starting preprocessing {}, {}", self.modified_graph.number_nodes, self.graph_ref.number_nodes);
        // create map of ranks
        for i in 0..(self.graph_ref.number_nodes as u32) {
            // contraction order heuristic: out-degree
            let curr_rank = calc_number_edges(i, &self.graph_ref) as usize;
            rank_map[curr_rank].push(i);
        }

        for i in 0..rank_map.len() {
            println!("i {} len {}", i, rank_map[i].len());
        }

        while rank < rank_map.len() {
            // select nodes C
            let c: &Vec<u32> = &rank_map[rank];
            println!("calculating rank {}, nodes to calc {}, number shortcuts added {}", rank, c.len(), self.shortcuts.len());

            // ink rank
            rank += 1;

            let mut modified_adj_array = self.modified_graph.adjacency_array_consider_removed_nodes(&self.removed_edges);

            // create shortcut set S and add them to new_e
            for i in c {
                self.find_shortcuts(*i, &modified_adj_array);
                // remove adjacent edges of i
                self.modified_graph.remove_edges_of_node(*i);
                self.removed_edges.insert(*i, true);
            }

            // final graph with added shortcuts
            self.shortcuts_start_indexes.insert(rank, self.modified_graph.edges.len());
            println!("new index: {}", self.modified_graph.edges.len());
            for s in &self.shortcuts {
                self.modified_graph.edges.push(s.edge);
                self.modified_graph.offsets[s.edge.source as usize] += 1;


            }
        }

        // save graph to disc
        let mut f = BufWriter::new(File::create(
            Path::new("/home/gin/Documents/UNI/master/FachpraktikumAlgorithms/osm-tasks-fachpraktikum-algorithms-ss2021/osm-tasks/finally.bin")).unwrap());
        if let Err(e) = bincode::serialize_into(&mut f, &self.modified_graph) {
            println!("Could not save graph to disk: {:?}", e);
        }
    }

    fn find_shortcuts(&mut self, node: u32, adj_array: &AdjacencyArray) {
        let mut dijkstra = DijkstraToMany::new(adj_array, node);
        self.contracted_nodes.insert(node, true);

        // get neighbors
        let neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(node);

        let mut neighbors = vec![];
        for i in (0..neighbors_and_distances.len()).step_by(2) {
            // ignore neighbors which have already been contracted
            if !self.contracted_nodes.contains_key(&neighbors_and_distances[i]) && neighbors_and_distances[i] != u32::MAX {
                neighbors.push(neighbors_and_distances[i]);
            }
        }
        if neighbors.len() <= 1 {
            return;
        }

        for j in 0..neighbors.len() {
            let targets: Vec<u32> = neighbors.iter().filter(|&&n| n != neighbors[j]).cloned().collect::<Vec<u32>>();
            dijkstra.change_source_node(neighbors[j]);
            if let Some(result) = dijkstra.find_route(targets) {
                let routes = result.0;
                let distances = result.1;
                let mut counter = 0;
                for route in routes {
                    // we are only interested in routes containing the node
                    if route.contains(&node) {
                        let edge = Edge {
                            source: route[0], target: route[route.len()-1],
                            distance: distances[counter]
                        };
                        // TODO consider addding shortcut in both directions
                        self.shortcuts.push( Shortcut {
                            replaced_edges: route,
                            edge
                        });
                    }
                    counter += 1;
                }
            }
        }
    }

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
                        previous_node: curr.node_id
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
                        previous_node: curr.node_id
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
    }
}

fn calc_number_edges(v: u32, graph: &GridGraph) -> u32 {
    if v > 0 {
        return graph.offsets[v as usize] - graph.offsets[v as usize - 1];
    }
    return graph.offsets[v as usize];
}
