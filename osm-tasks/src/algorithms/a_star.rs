use std::cmp::Ordering;
use std::collections::BinaryHeap;
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::grid_graph::GridGraph;
use crate::model::priority_heap_item::PriorityHeapItem;

pub(crate) struct AStar<'a> {
    adj_ref: AdjacencyArray,
    graph_ref: &'a GridGraph,
    heap: BinaryHeap<PriorityHeapItem>,
    distances: Vec<u32>,
    previous_nodes: Vec<u32>,
    source_node: u32,
    amount_nodes_popped: u32,
}

impl<'a> AStar<'a> {
    pub fn new(grid_graph: &GridGraph, source_node: u32) -> AStar {
        //println!("New dijkstra instance with source node {}", source_node);
        let number_of_nodes = grid_graph.nodes.len();
        let mut heap = BinaryHeap::with_capacity(number_of_nodes);
        let distances = vec![u32::MAX; number_of_nodes];
        let previous_nodes = vec![u32::MAX; number_of_nodes];
        heap.push(PriorityHeapItem {
            node_id: source_node,
            distance: 0,
            priority: 0,
            previous_node: source_node,
        });
        return AStar { adj_ref: grid_graph.adjacency_array(), graph_ref: grid_graph, heap, distances, previous_nodes, source_node, amount_nodes_popped: 0 };
    }

    pub fn find_route(&mut self, destination_node: u32) -> Option<(Vec<u32>, u32, u32)> {
        self.a_star(&destination_node);
        if self.distances[destination_node as usize] != u32::MAX {
            Some((self.traverse_route(&destination_node), self.distances[destination_node as usize], self.amount_nodes_popped))
        } else {
            None
        }
    }

    fn a_star(&mut self, destination_node: &u32) {
        loop {
            if let Some(heap_element) = self.heap.pop() {
                self.amount_nodes_popped += 1;
                //println!("Popped element from heap {}", heap_element);
                if heap_element.distance >= self.distances[heap_element.node_id as usize] {
                    //println!("Skipping heap element {:?} because lower distance is already set: {}", heap_element, self.distances[heap_element.node_id as usize]);
                    continue;
                }
                self.previous_nodes[heap_element.node_id as usize] = heap_element.previous_node;
                self.distances[heap_element.node_id as usize] = heap_element.distance;
                let neighbors_and_distances = self.adj_ref.get_neighbors_of_node_and_distances(heap_element.node_id);
                //println!("distance {}", dist_to_dest);
                for i in (0..neighbors_and_distances.len()).step_by(2) {
                    let next_node = neighbors_and_distances[i];
                    let next_node_distance = neighbors_and_distances[i + 1] as u64;

                    // heuristic
                    let heuristic = self.graph_ref.get_distance(next_node, *destination_node);

                    if self.distances[next_node as usize] == u32::MAX {
                        //println!("add edge form {} to {} with dist {}", heap_element.node_id, next_node, next_node_distance);
                        self.heap.push(PriorityHeapItem {
                            node_id: next_node,
                            distance: (next_node_distance as u32) + heap_element.distance,
                            priority: next_node_distance + (heap_element.distance as u64) + heuristic,
                            previous_node: heap_element.node_id,
                        });
                    }
                }
                if *destination_node == heap_element.node_id {
                    // found dest
                    break;
                }
            } else {
                println!("Heap is empty but dest node not found. src {}, dest {}", self.source_node, destination_node);
                return;
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
