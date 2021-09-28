/// Compact GridGraph which uses adjacency list with primitive types to store the graph
pub struct AdjacencyArray {
    edges_and_distances_offsets: Vec<u32>,
    edges_and_distances: Vec<u32>
}

impl AdjacencyArray {
    pub fn new(edges_and_distances_offsets: Vec<u32>, edges_and_distances: Vec<u32>) -> AdjacencyArray {
        AdjacencyArray { edges_and_distances_offsets, edges_and_distances }
    }

    pub fn edges_and_distances_offsets(&self) -> &Vec<u32>{
        &self.edges_and_distances_offsets
    }
    pub fn edges_and_distances(&self) -> &Vec<u32> {
        &self.edges_and_distances
    }
    pub(crate) fn get_neighbors_of_node_and_distances(&self, node: u32) -> &[u32] {
        &self.edges_and_distances[(self.edges_and_distances_offsets[node as usize] as usize)..(self.edges_and_distances_offsets[node as usize + 1] as usize)]
    }
    pub(crate) fn get_nodes_count(&self) -> u32 {
        self.edges_and_distances_offsets.len() as u32 - 1
    }

    /*
    pub(crate) fn remove_neighbors_edges(&mut self, node: u32) {
        for i in (self.edges_and_distances_offsets[node as usize] as usize)..(self.edges_and_distances_offsets[node as usize + 1] as usize) {
            self.edges_and_distances[i] = u32::MAX;
        }
    } */
}
