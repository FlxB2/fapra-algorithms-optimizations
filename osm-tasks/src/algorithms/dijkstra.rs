use std::collections::BinaryHeap;
use crate::model::adjacency_array::AdjacencyArray;
use crate::model::heap_item::HeapItem;

#[allow(dead_code)]
pub(crate) struct DummyGraph {
    offsets: Vec<u32>,
    edges: Vec<u32>,
}

#[allow(dead_code)]
impl DummyGraph {
    pub(crate) fn init() -> DummyGraph {
        let offsets_old = vec![0, 2, 5, 8, 11, 16, 19, 22, 24];
        let edges_target = vec![1, 4, 0, 2, 4, 1, 3, 4, 2, 4, 6, 0, 1, 2, 3, 5, 4, 6, 7, 3, 5, 7, 5, 6];
        let edges_distance = vec![2, 1, 2, 2, 1, 2, 1, 2, 1, 3, 2, 1, 1, 2, 3, 1, 1, 3, 1, 2, 3, 1, 1, 1];
        let offsets: Vec<u32> = offsets_old.into_iter().map(|i| { i * 2 }).collect();
        let mut edges = Vec::with_capacity(edges_target.len() * 2);
        for i in 0..edges_target.len() {
            edges.push(edges_target[i]);
            edges.push(edges_distance[i]);
        }
        DummyGraph {
            offsets,
            edges,
        }
    }

    fn get_neighbors_of_node_and_distances(&self, node: u32) -> &[u32] {
        let start_offset = self.offsets[node as usize] as usize;
        let next_node_offset = self.offsets[node as usize + 1] as usize;
        &self.edges[start_offset..next_node_offset]
    }

    fn get_nodes_count(& self) -> u32 {
        self.offsets.len() as u32 - 1
    }
}

pub(crate) struct Dijkstra {
    graph_ref: AdjacencyArray,
    heap: BinaryHeap<HeapItem>,
    distances: Vec<u32>,
    previous_nodes: Vec<u32>,
    source_node: u32,
    amount_nodes_popped: u32,
}

impl Dijkstra {
    pub fn new(graph: AdjacencyArray, source_node: u32) -> Dijkstra {
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
        return Dijkstra { graph_ref: graph, heap, distances, previous_nodes, source_node, amount_nodes_popped: 0 };
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

    pub fn find_route(&mut self, destination_node: u32) -> Option<(Vec<u32>, u32, u32)> {
        /* disable caching
        if self.distances[destination_node as usize] != u32::MAX {
            return Some((self.traverse_route(&destination_node), self.distances[destination_node as usize]));
        } */
        self.dijkstra(&destination_node);
        if self.distances[destination_node as usize] != u32::MAX {
            Some((self.traverse_route(&destination_node), self.distances[destination_node as usize], self.amount_nodes_popped))
        } else {
            None
        }
    }

    fn dijkstra(&mut self, destination_node: &u32) {
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
                let neighbors_and_distances = self.graph_ref.get_neighbors_of_node_and_distances(heap_element.node_id);
                for i in (0..neighbors_and_distances.len()).step_by(2) {
                    let next_node = neighbors_and_distances[i];
                    let next_node_distance = neighbors_and_distances[i + 1];
                    if self.distances[next_node as usize] == u32::MAX {
                        //println!("add edge form {} to {} with dist {}", heap_element.node_id, next_node, next_node_distance);
                        self.heap.push(HeapItem {
                            node_id: next_node,
                            distance: next_node_distance + heap_element.distance,
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
