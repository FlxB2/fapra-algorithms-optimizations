use std::collections::{BinaryHeap, HashMap};
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::heap_item::HeapItem;
use crate::model::grid_graph::GridGraph;

pub(crate) struct WitnessSearch<'a> {
    graph_ref: &'a AdjacencyArray,
    heap: BinaryHeap<HeapItem>,
    distances: Vec<u32>,
    previous_nodes: Vec<u32>,
    source_node: u32,
    amount_nodes_popped: u32,
    removed_nodes: &'a HashMap<u32,bool>,
}

// basically one to many dijkstra
impl<'a> WitnessSearch<'a> {
    pub fn new(graph: &'a AdjacencyArray, source_node: u32, removed_nodes: &'a HashMap<u32,bool>) -> WitnessSearch<'a> {
        //println!("New dijkstra instance with source node {}", source_node);
        let number_of_nodes = graph.get_nodes_count() as usize;
        let mut heap = BinaryHeap::with_capacity(number_of_nodes);
        let distances = vec![u32::MAX; number_of_nodes];
        let previous_nodes = vec![u32::MAX; number_of_nodes];
        heap.push(HeapItem {
            node_id: source_node,
            distance: 0,
            previous_node: source_node,
        });
        return WitnessSearch { graph_ref: graph, removed_nodes, heap, distances, previous_nodes, source_node, amount_nodes_popped: 0 };
    }

    pub fn change_source_node(&mut self, source_node: u32) {
        if source_node == self.source_node {
            return;
        }
        //println!("Reinitialized dijkstra for new source node {}", source_node);
        self.source_node = source_node;
        self.heap.clear();
        self.heap.push(HeapItem {
            node_id: source_node,
            distance: 0,
            previous_node: source_node,
        });
        self.distances.fill(u32::MAX);
        self.previous_nodes.fill(u32::MAX);
    }

    pub fn find_route(&mut self, destination_nodes: &[u32]) -> Option<(Vec<Vec<u32>>, Vec<u32>, u32)> {
        /* disable caching
        if self.distances[destination_node as usize] != u32::MAX {
            return Some((self.traverse_route(&destination_node), self.distances[destination_node as usize]));
        } */
        let result = self.dijkstra(destination_nodes);
        if result.1.len() > 0 {
            Some((result.0, result.1, self.amount_nodes_popped))
        } else {
            None
        }
    }

    fn dijkstra(&mut self, destination_nodes: &[u32]) -> (Vec<Vec<u32>>, Vec<u32>) {
        let mut results: Vec<Vec<u32>> = vec![];
        let mut distances: Vec<u32> = vec![];
        loop {
            if let Some(heap_element) = self.heap.pop() {
                if heap_element.distance >= self.distances[heap_element.node_id as usize] {
                    //println!("Skipping heap element {:?} because lower distance is already set: {}", heap_element, self.distances[heap_element.node_id as usize]);
                    continue;
                }
                self.previous_nodes[heap_element.node_id as usize] = heap_element.previous_node;
                self.distances[heap_element.node_id as usize] = heap_element.distance;
                let neighbors_and_distances = self.graph_ref.get_neighbors_of_node_and_distances(heap_element.node_id);
                for i in (0..neighbors_and_distances.len()).step_by(2) {
                    let next_node = neighbors_and_distances[i];
                    let next_node_distance = neighbors_and_distances[i + 1];

                    if self.removed_nodes.contains_key(&next_node) || next_node == u32::MAX || next_node_distance == u32::MAX {
                        continue;
                    }

                    if self.distances[next_node as usize] == u32::MAX {
                        //println!("add edge form {} to {} with dist {}", heap_element.node_id, next_node, next_node_distance);
                        self.heap.push(HeapItem {
                            node_id: next_node,
                            distance: next_node_distance + heap_element.distance,
                            previous_node: heap_element.node_id,
                        });
                    }
                }
                if destination_nodes.contains(&heap_element.node_id) {
                    // found dest
                    results.push(self.traverse_route(&heap_element.node_id));
                    distances.push(self.distances[heap_element.node_id as usize]);
                    if results.len() == destination_nodes.len() {
                        return (results, distances);
                    }
                }
            } else {
                //println!("Heap is empty but dest node not found. src {}", self.source_node);
                return (vec![], vec![]);
            }
        }
    }

    fn traverse_route(&self, destination_node: &u32) -> Vec<u32> {
        let mut previous_node = self.previous_nodes[*destination_node as usize];
        let mut nodes = vec![*destination_node];
        while previous_node != self.source_node {
            nodes.push(previous_node);
            previous_node = self.previous_nodes[previous_node as usize];
        }
        nodes.push(self.source_node);
        nodes.reverse();
        return nodes;
    }
}
