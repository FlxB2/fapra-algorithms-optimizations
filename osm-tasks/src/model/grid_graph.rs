/*
simple grid graph representation following a classic adjacency list
 */

use std::f64::consts::PI;
use std::f64;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::time::Instant;
use crate::config::Config;
use crate::model::adjacency_array::AdjacencyArray;
use crate::algorithms::polygon_test::PointInPolygonTest;
use std::collections::HashMap;

/// Returns the upper bound of the number of nodes in this graph.
pub fn get_maximum_number_of_nodes() -> usize {
    Config::global().number_of_nodes() as usize
}

#[derive(Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub source: u32,
    pub target: u32,
    pub(crate) distance: u32,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        other.source == self.source && other.target == self.target
    }
}

impl Eq for Edge {}

#[derive(Clone, Copy, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Node {
    pub lat: f64,
    pub lon: f64,
}

impl Into<(f64, f64)> for Node {
    fn into(self) -> (f64, f64) {
        (self.lon, self.lat)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GridGraph {
    pub number_edges: i64,
    pub number_nodes: i64,
    // index equals node id of source
    pub edges: Vec<Vec<Edge>>,
    // index equals node id
    pub nodes: Vec<Node>,
}

impl GridGraph {
    // Generates an adjacency array representation of the edges of this graph
    pub fn adjacency_array(&self) -> AdjacencyArray {
        let mut edges_and_distances = vec![u32::MAX; (self.number_edges * 2) as usize];
        let mut offsets = vec![u32::MAX; self.number_nodes as usize + 1];
        let mut counter = 0;
        let mut prev_offset = 0;
        self.edges.iter().enumerate().for_each(|(i, edges)| {
            // for each node add all edges to adj array
            for j in 0..edges.len() {
                edges_and_distances[counter * 2] = edges[j].target;
                edges_and_distances[counter * 2 + 1] = edges[j].distance;
                counter += 1;
            }

            offsets[i] = prev_offset;
            prev_offset += self.edges[i].len() as u32 * 2
        });
        offsets[self.number_nodes as usize] = prev_offset;
        return AdjacencyArray::new(offsets, edges_and_distances);
    }

    pub fn remove_edges_of_node(&mut self, v: u32) {
        self.edges[v as usize] = vec![];
    }

    pub fn add_new_edge(&mut self, edge: Edge) {
        self.edges[edge.source as usize].push(edge)
    }

    // distance in km, should be sufficient
    pub fn get_distance(&self, node1: u32, node2: u32) -> u64 {
        calculate_length_between_points_on_sphere(&self.nodes[node1 as usize], &self.nodes[node2 as usize]) as u64
    }

    pub fn default() -> GridGraph {
        GridGraph {
            number_edges: 0,
            number_nodes: 0,
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }
    pub fn new(polygon_test: &PointInPolygonTest, number_nodes: usize) -> GridGraph {
        // mapping from virtual nodes indices (0..NUMBER_NODES) (includes nodes inside of polygons) to the actual nodes of the grid (includes only nodes of the graph)
        let start_time = Instant::now();
        let maximum_number_of_nodes = number_nodes;
        let mut virtual_nodes_to_index: Vec<Option<u32>> = vec![None; maximum_number_of_nodes];
        let mut number_virtual_nodes: usize = 0;
        let mut number_graph_nodes: usize = 0;
        let mut nodes = vec![Node { lat: 0.0, lon: 0.0 }; maximum_number_of_nodes];
        let mut edges: Vec<Vec<Edge>> = vec![Vec::with_capacity(8); maximum_number_of_nodes];

        // algorithm taken from here https://www.cmu.edu/biolphys/deserno/pdf/sphere_equi.pdf
        // number of nodes is only very close not equal to NUMBER_NODES
        let pi = PI as f64;
        let radius_earth: f64 = 1.0; // in km
        let a: f64 = 4.0 * pi * (radius_earth.powf(2.0) / maximum_number_of_nodes as f64);
        let d: f64 = a.sqrt();
        let m_theta = (pi / d).round() as i32;
        let d_theta: f64 = pi / (m_theta as f64);
        let d_phi: f64 = a / d_theta;
        let mut m_phi;
        let mut number_azimuth_steps_last_round = 0;
        let mut number_virtual_nodes_before_last_round = 0;
        // calculated in rad!!
        for m in (0..m_theta).rev() {
            if ((number_virtual_nodes as f64 / maximum_number_of_nodes as f64) * 100.0).ceil() as i32 > ((number_virtual_nodes_before_last_round as f64 / maximum_number_of_nodes as f64) * 100.0).ceil() as i32 {
                println!("Generating graph: {}%", ((number_virtual_nodes as f64 / maximum_number_of_nodes as f64) * 100.0).ceil() as i32);
            }
            let polar = pi * ((m as f64) + 0.5) / (m_theta as f64);
            m_phi = (2.0 * pi * (polar).sin() / d_phi).round() as i32;
            let number_azimuth_steps_this_round = (0..m_phi).len();
            let number_virtual_nodes_at_start_of_this_round = number_virtual_nodes;
            let lat = polar * (180.0 / pi) - 90.0;
            // Do point in polygon test in parallel and collect results
            let nodes_to_place: Vec<(i32, Option<Node>)> = (0..m_phi).into_par_iter().map(|n| {
                let azimuthal = 2.0 * pi * (n as f64) / (m_phi as f64);

                // convert rad to degrees and lon = polar - 90; lat = azimuthal-180
                let lon = azimuthal * (180.0 / pi) - 180.0;

                if polygon_test.check_intersection(*&(lon, lat)) {
                    (n, None)
                } else {
                    let source_node = Node { lat, lon };
                    (n, Some(source_node))
                }
            }).collect();
            let mut last_node_mid_top_node_orientation = NodeOrientation::MID;
            nodes_to_place.into_iter().for_each(|(n, source_node_option)| {
                let n_float = n as f64;
                if let Some(source_node) = source_node_option {
                    if number_virtual_nodes < maximum_number_of_nodes {
                        nodes[number_graph_nodes] = source_node;
                        virtual_nodes_to_index[number_virtual_nodes] = Some(number_graph_nodes as u32);
                        if number_azimuth_steps_last_round > 3 {

                            // Use a rule of three like approach to calculate the virtual node index of the nearest nodes above (in the last round)
                            let offset_float = n_float + (number_azimuth_steps_last_round as f64) * ((number_azimuth_steps_this_round as f64 - n_float) / (number_azimuth_steps_this_round as f64));

                            let virtual_index_top_right_node = number_virtual_nodes - offset_float.floor() as usize;
                            let virtual_index_top_left_node = number_virtual_nodes - offset_float.ceil() as usize;
                            if virtual_index_top_left_node == virtual_index_top_right_node {
                                // node is exactly above -> also add edges to the nodes right and left
                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node - 1)]);
                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node + 1)]);
                            } else {
                                // add edges to the three nearest nodes above
                                let distance_right = add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                                let distance_left = add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node)]);
                                // insert third node on the other side of the nearest node
                                match (distance_right, distance_left) {
                                    (None, Some(_)) => { add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]); }
                                    (Some(_), None) => { add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node + 1)]); }
                                    (Some(dst_right), Some(dst_left)) => {
                                        if dst_left != dst_right {
                                            if dst_left > dst_right {
                                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node + 1)]);
                                                // check if the orientation has flipped from left to right
                                                if last_node_mid_top_node_orientation == NodeOrientation::LEFT {
                                                    // insert extra edges crossed over the gap,so that no gap is produced
                                                    add_extra_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]);
                                                    if let Some(left_neighbor_index) = &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_at_start_of_this_round, &(m_phi as usize), number_virtual_nodes + (m_phi - 1) as usize)] {
                                                        add_extra_edge(&mut edges, &nodes, *left_neighbor_index as usize, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                                                    }
                                                }
                                                last_node_mid_top_node_orientation = NodeOrientation::RIGHT;
                                            } else {
                                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]);
                                                last_node_mid_top_node_orientation = NodeOrientation::LEFT;
                                            }
                                        } else {
                                            if last_node_mid_top_node_orientation == NodeOrientation::LEFT {
                                                // insert extra edges crossed over the gap,so that no gap is produced
                                                add_extra_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]);
                                                if let Some(left_neighbor_index) = &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_at_start_of_this_round, &(m_phi as usize), number_virtual_nodes + (m_phi - 1) as usize)] {
                                                    add_extra_edge(&mut edges, &nodes, *left_neighbor_index as usize, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                                                }
                                            }
                                        }
                                    }
                                    (_, _) => {}
                                }
                            }
                            // add edge to neighbor
                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_at_start_of_this_round, &(m_phi as usize), number_virtual_nodes + (m_phi - 1) as usize)]);
                            if n == m_phi - 1 && number_virtual_nodes_at_start_of_this_round > 1 {
                                // edge between last node in round and first node of this round, must be done manually,
                                // because the last node does not yet exist, if the first node tries to insert the edge
                                add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[number_virtual_nodes_at_start_of_this_round]);
                            }
                        }
                        number_graph_nodes += 1;
                    }
                }
                number_virtual_nodes += 1;
            });
            number_azimuth_steps_last_round = number_azimuth_steps_this_round;
            number_virtual_nodes_before_last_round = number_virtual_nodes_at_start_of_this_round;
        }
        // flatten edge array to 1 dimension and calculate offsets
        /*
        let mut offsets = Vec::with_capacity(edges.len() + 1);
        offsets.push(0);
        let mut last_offset = 0;
        for i in 0..number_graph_nodes {
            last_offset = edges[i].len() as u32 + last_offset;
            offsets.push(last_offset);
        }
        let flattened_edges: Vec<Edge> = edges.concat(); */

        let mut number_edges = 0;
        edges.iter().for_each(|e| number_edges += e.len());

        println!("number even distributed nodes {}", number_virtual_nodes);
        println!("number placed nodes {}", number_graph_nodes);
        println!("number edges {}", number_edges);

        // Remove unset nodes from nodes array
        nodes.truncate(number_graph_nodes);
        edges.truncate(number_graph_nodes);
        println!("Generated graph in {} seconds", start_time.elapsed().as_secs());
        GridGraph {
            number_edges: number_edges as i64,
            number_nodes: number_graph_nodes as i64,
            edges,
            nodes,
        }
    }
}

fn add_edge(edges: &mut Vec<Vec<Edge>>, nodes: &Vec<Node>, node1_idx: usize, node2_idx_option: &Option<u32>) -> Option<f64> {
    if let Some(node2_idx) = node2_idx_option {
        // target node is part of the graph
        let distance = calculate_length_between_points_on_sphere(&nodes[node1_idx as usize], &nodes[*node2_idx as usize]);
        edges[node1_idx].push(Edge { source: node1_idx as u32, target: *node2_idx, distance: distance as u32 });
        edges[*node2_idx as usize].push(Edge { source: *node2_idx, target: node1_idx as u32, distance: distance as u32 });
        return Some(distance);
    }
    return None;
}

// like add_edge but checks if the edge is already present before inserting the edge
fn add_extra_edge(edges: &mut Vec<Vec<Edge>>, nodes: &Vec<Node>, node1_idx: usize, node2_idx_option: &Option<u32>) -> Option<f64> {
    if let Some(node2_idx) = node2_idx_option {
        // target node is part of the graph
        let distance = calculate_length_between_points_on_sphere(&nodes[node1_idx as usize], &nodes[*node2_idx as usize]);
        // check for duplicates
        if !edges[node1_idx].iter().any(|e| { e.target == *node2_idx }) {
            edges[node1_idx].push(Edge { source: node1_idx as u32, target: *node2_idx, distance: distance as u32 });
        }
        if !edges[*node2_idx as usize].iter().any(|e| { e.target == node1_idx as u32 }) {
            edges[*node2_idx as usize].push(Edge { source: *node2_idx, target: node1_idx as u32, distance: distance as u32 });
        }
        return Some(distance);
    }
    return None;
}

fn calc_index_modulo(round_start_index: &usize, nodes_in_rounds: &usize, index_usize: usize) -> usize {
    let mut index = index_usize as isize;
    index = index - *round_start_index as isize;
    index = index + *nodes_in_rounds as isize;
    let new_index = (index % *nodes_in_rounds as isize) + *round_start_index as isize;
    new_index as usize
}

const EARTH_RADIUS: f64 = 6_378_137_f64; // earth radius in meters

fn calculate_length_between_points_on_sphere(node1: &Node, node2: &Node) -> f64 {
    distance(node1.lon, node1.lat, node2.lon, node2.lat)
}

// expects lat/lon in degrees
pub fn distance(lon1_deg: f64, lat1_deg: f64, lon2_deg: f64, lat2_deg: f64) -> f64 {
    let lat1 = lat1_deg.to_radians();
    let lat2 = lat2_deg.to_radians();
    let lon1 = lon1_deg.to_radians();
    let lon2 = lon2_deg.to_radians();
    let dlat_sin = ((lat2 - lat1) / 2.0).sin();
    let dlon_sin = ((lon2 - lon1) / 2.0).sin();
    let a = dlat_sin.powf(2.0) + lat1.cos() * lat2.cos() * dlon_sin.powf(2.0);
    let c = 2.0 * (a.sqrt()).asin();
    return EARTH_RADIUS * c;
}

#[derive(Clone, PartialEq, Eq, Copy)]
enum NodeOrientation {
    LEFT,
    RIGHT,
    MID,
}
