use crate::grid_graph::{distance, Node};

#[derive(Clone, Copy, Debug)]
struct NodeWithId {
    lon: f64,
    lat: f64,
    id: u32,
}

impl NodeWithId {
    fn distance_to(&self, node: &Node) -> f64 {
        distance(self.lon, self.lat, node.lon, node.lat)
    }

    fn min<'a>(node1: Option<(&'a NodeWithId, f64)>, node2: Option<(&'a NodeWithId, f64)>) -> Option<(&'a NodeWithId, f64)> {
        match (node1, node2) {
            (None, None) => None,
            (Some(node), None) | (None, Some(node)) => Some(node),
            (Some((_, dst1)), Some((_, dst2))) => {
                if dst1 > dst2 {
                    node2
                } else {
                    node1
                }
            }
        }
    }
}

pub struct NearestNeighbor {
    grid: Vec<Vec<NodeWithId>>,
}

const X_SIZE: usize = 100;
const Y_SIZE: usize = 100;

impl NearestNeighbor {
    pub fn new(nodes: &Vec<Node>) -> NearestNeighbor {
        let mut grid = vec![Vec::new(); X_SIZE * Y_SIZE];
        for i in 0..nodes.len() {
            let node = &nodes[i];
            grid[NearestNeighbor::get_cell_for_node(node)].push(NodeWithId { id: i as u32, lon: node.lon, lat: node.lat });
        }
        NearestNeighbor { grid }
    }

    pub fn find_nearest_neighbor(&self, node: &Node) -> u32 {
        // first check the cell itself and all eight cells around
        let node_index = NearestNeighbor::get_cell_for_node(node);
        let mut nearest_node_and_distance = self.find_nearest_neighbor_in_cell(node_index, node);
        for i in 1..X_SIZE.max(Y_SIZE) {
            let (new_nearest_node_and_distance, radius) = self.find_nearest_neighbor_for_radius(i, node_index, node);
            nearest_node_and_distance = NodeWithId::min(nearest_node_and_distance, new_nearest_node_and_distance);
            if let Some((nearest_node, distance)) = nearest_node_and_distance {
                if distance <= radius {
                    return nearest_node.id;
                }
            }
        }
        panic!("Invariant violated: Could not find nearest neighbor node for coords {} {}", node.lon, node.lat);
    }

    /// returns the nearest node and teh distance to this node as well as the used radius for this query
    fn find_nearest_neighbor_for_radius(&self, distance_to_center: usize, center_cell: usize, node: &Node) -> (Option<(&NodeWithId, f64)>, f64) {
        let mut nearest_node_and_distance: Option<(&NodeWithId, f64)> = None;
        let mut radius = f64::MAX;
        let (center_x, center_y) = NearestNeighbor::get_x_y_for_index(center_cell);
        for y in vec![center_y as isize - distance_to_center as isize, center_y as isize + distance_to_center as isize] {
            if y < 0 || y >= Y_SIZE as isize - 1 { continue; }
            for x in (center_x as isize - distance_to_center as isize)..(center_x as isize + distance_to_center as isize) {
                let x_mod = ((x + X_SIZE as isize) % X_SIZE as isize) as usize; // X_SIZE that the result can not be negative
                nearest_node_and_distance = NodeWithId::min(nearest_node_and_distance, self.find_nearest_neighbor_in_cell(NearestNeighbor::get_index_for_x_y(x_mod, y as usize), node));
                let (cell_midpoint_lon, cell_midpoint_lat) = NearestNeighbor::calc_mid_point_of_cell(x_mod, y as usize);
                radius = radius.min(distance(node.lon, node.lat, cell_midpoint_lon, cell_midpoint_lat));
            }
        }
        for x in vec![center_x as isize - distance_to_center as isize, center_x as isize + distance_to_center as isize] {
            let x_mod = ((x + X_SIZE as isize) % X_SIZE as isize) as usize; // X_SIZE that the result can not be negative
            // the first and last element has been taken into account when iterating the x lines
            for y in (center_y as isize - distance_to_center as isize + 1)..(center_y as isize + distance_to_center as isize - 1) {
                if y < 0 || y >= Y_SIZE as isize - 1 { continue; }
                nearest_node_and_distance = NodeWithId::min(nearest_node_and_distance, self.find_nearest_neighbor_in_cell(NearestNeighbor::get_index_for_x_y(x_mod, y as usize), node));
                let (cell_midpoint_lon, cell_midpoint_lat) = NearestNeighbor::calc_mid_point_of_cell(x_mod, y as usize);
                radius = radius.min(distance(node.lon, node.lat, cell_midpoint_lon, cell_midpoint_lat));
            }
        }
        (nearest_node_and_distance, if radius == f64::MAX { 0.0 } else { radius })
    }

    fn find_nearest_neighbor_in_cell(&self, cell_index: usize, node: &Node) -> Option<(&NodeWithId, f64)> {
        let mut nearest_node = None;
        let mut distance_to_nearest_node = f64::MAX;
        for x in &self.grid[cell_index] {
            let distance = x.distance_to(node);
            if distance_to_nearest_node > distance {
                distance_to_nearest_node = distance;
                nearest_node = Some(x);
            }
        }
        if nearest_node.is_none() {
            None
        } else {
            Some((nearest_node.unwrap(), distance_to_nearest_node))
        }
    }

    fn calc_mid_point_of_cell(x: usize, y: usize) -> (f64, f64) {
        let (lon1_deg, lat1_deg) = NearestNeighbor::get_coords_of_x_y(x, y);
        (lon1_deg + (x as f64 / X_SIZE as f64) * 0.5, lat1_deg + (y as f64 / Y_SIZE as f64) * 0.5)
    }

    fn get_coords_of_x_y(x: usize, y: usize) -> (f64, f64) {
        let p_x = (x as f64 / X_SIZE as f64) * 360.0;
        let p_y = (y as f64 / Y_SIZE as f64) * 180.0;
        (p_x - 180.0, p_y as f64 - 90.0)
    }

    #[inline]
    fn get_index_for_x_y(x: usize, y: usize) -> usize {
        x + (y * X_SIZE)
    }

    #[inline]
    fn get_x_y_for_index(index: usize) -> (usize, usize) {
        let y = index / X_SIZE;
        let x = index - (y * X_SIZE);
        (x, y)
    }

    fn get_cell_for_node(node: &Node) -> usize {
        let lon = if node.lon >= 180.0 { -180.0 } else { node.lon };
        let x = (((lon + 180.0) / 360.0) * X_SIZE as f64) as usize;
        let y = (((node.lat + 90.0)/  180.0) * Y_SIZE as f64) as usize;
        NearestNeighbor::get_index_for_x_y(x, y)
    }
}