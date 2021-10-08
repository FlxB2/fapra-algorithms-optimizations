use std::collections::{BinaryHeap, HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::GridGraph;
use crate::model::cn_model::{CNMetadata, Shortcut};
use crate::model::priority_heap_item::PriorityHeapItem;
use crate::model::heap_item::HeapItem;
use crate::algorithms::unwrap_shortcuts::unwrap_shortcuts;

pub(crate) struct CNBdDijkstra<'a> {
    meta: &'a CNMetadata,
    graph_ref: &'a GridGraph,
    forward_heap: BinaryHeap<PriorityHeapItem>,
    backward_heap: BinaryHeap<PriorityHeapItem>,
    forward_distances: Vec<u32>,
    backward_distances: Vec<u32>,
    forward_previous_nodes: Vec<u32>,
    backward_previous_nodes: Vec<u32>,
    source_node: u32,
    destination_node: u32,
    amount_nodes_popped_forward: usize,
    amount_nodes_popped_backward: usize,
    mu: u32,
    meeting_node: u32,
}

impl<'a> CNBdDijkstra<'a> {
    pub fn new(meta: &CNMetadata, source_node: u32) -> CNBdDijkstra {
        let number_of_nodes = meta.graph.nodes.len() as usize;
        let forward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let backward_heap = BinaryHeap::with_capacity(number_of_nodes);
        let forward_distances = vec![u32::MAX; number_of_nodes];
        let backward_distances = vec![u32::MAX; number_of_nodes];
        let forward_previous_nodes = vec![u32::MAX; number_of_nodes];
        let backward_previous_nodes = vec![u32::MAX; number_of_nodes];

        return CNBdDijkstra {
            meta,
            graph_ref: &meta.graph,
            forward_heap,
            backward_heap,
            forward_distances,
            backward_distances,
            forward_previous_nodes,
            backward_previous_nodes,
            destination_node: 0,
            source_node,
            amount_nodes_popped_forward: 0,
            amount_nodes_popped_backward: 0,
            mu: u32::MAX,
            meeting_node: u32::MAX,
        };
    }

    pub fn find_route(&mut self, destination_node: u32) -> Option<(Vec<u32>, u32, u32)> {
        let meeting_node = self.bd_dijkstra(self.source_node, destination_node);
        self.destination_node = destination_node;

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

        println!("starting to unwrap shortcuts");
        let complete_route = unwrap_shortcuts(&route, &self.meta.get_shortcuts);

        Some((complete_route,
              self.forward_distances[meeting_node as usize] + self.backward_distances[meeting_node as usize],
              (self.amount_nodes_popped_forward + self.amount_nodes_popped_backward) as u32))
    }

    fn bd_dijkstra(&mut self, source_node: u32, destination_node: u32) -> u32 {
        let adj_array = self.graph_ref.adjacency_array();
        self.meeting_node = u32::MAX;
        self.mu = u32::MAX;

        self.backward_heap.push(PriorityHeapItem {
            node_id: destination_node,
            distance: 0,
            priority: 0,
            previous_node: destination_node,
        });
        self.forward_heap.push(PriorityHeapItem {
            node_id: source_node,
            distance: 0,
            priority: 0,
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
            self.amount_nodes_popped_forward += 1;
            let mut neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(curr.node_id);

            // declared outside so it lives long enough
            let mut new_neighbors_and_distances = vec![];
            if let Some(shortcuts) = self.meta.get_shortcuts.get(&curr.node_id) {
                for s in shortcuts {
                    new_neighbors_and_distances.push(s.edge.target);
                    new_neighbors_and_distances.push(s.edge.distance);
                }
                neighbors_and_distances = new_neighbors_and_distances.as_slice();
            }

            // iterate over children
            for i in (0..neighbors_and_distances.len()).step_by(2) {
                let neighbor = neighbors_and_distances[i];
                let neighbor_distance = neighbors_and_distances[i + 1];

                let mut heuristic = 0;
                let score = curr.distance + neighbor_distance;
                let mut priority = score as u64;

                if self.forward_distances[neighbor as usize] == u32::MAX || self.forward_distances[neighbor as usize] > score {
                    // we did not encounter this node before
                    self.forward_previous_nodes[neighbor as usize] = curr.node_id;
                    self.forward_distances[neighbor as usize] = score;
                    self.forward_heap.push(PriorityHeapItem {
                        distance: score,
                        node_id: neighbor,
                        priority,
                        previous_node: curr.node_id,
                    });
                    self.update_best_path_forward(neighbor as usize, score, heuristic);
                }
            }
        }
    }

    fn expand_backward(&mut self, adj_array: &AdjacencyArray) {
        let current = self.backward_heap.pop();
        if let Some(curr) = current {
            self.amount_nodes_popped_backward += 1;
            let mut neighbors_and_distances = adj_array.get_neighbors_of_node_and_distances(curr.node_id);

            // declared outside so it lives long enough
            let mut new_neighbors_and_distances = vec![];
            if let Some(shortcuts) = self.meta.get_shortcuts.get(&curr.node_id) {
                for s in shortcuts {
                    new_neighbors_and_distances.push(s.edge.target);
                    new_neighbors_and_distances.push(s.edge.distance);
                }
                neighbors_and_distances = new_neighbors_and_distances.as_slice();
            }


            // iterate over children
            for i in (0..neighbors_and_distances.len()).step_by(2) {
                let neighbor = neighbors_and_distances[i];
                let neighbor_distance = neighbors_and_distances[i + 1];

                let mut heuristic = 0;
                let score = curr.distance + neighbor_distance;
                let mut priority = score as u64;

                if self.backward_distances[neighbor as usize] == u32::MAX || self.backward_distances[neighbor as usize] > score {
                    // we did not encounter this node before
                    self.backward_previous_nodes[neighbor as usize] = curr.node_id;
                    self.backward_distances[neighbor as usize] = score;
                    self.backward_heap.push(PriorityHeapItem {
                        distance: score,
                        node_id: neighbor,
                        priority,
                        previous_node: curr.node_id,
                    });
                    self.update_best_path_backward(neighbor as usize, score, heuristic);
                }
            }
        }
    }

    fn update_best_path_forward(&mut self, neighbor: usize, score: u32, heuristic: u32) -> bool {
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

    fn update_best_path_backward(&mut self, neighbor: usize, score: u32, heuristic: u32) -> bool {
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
