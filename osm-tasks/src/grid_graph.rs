/*
simple grid graph representation following a classic adjacency list
 */

use std::f32::consts::PI;
use std::f32;
use core::num::FpCategory::Nan;
use serde::{Deserialize, Serialize};
use crate::polygon_test::PointInPolygonTest;
use rayon::prelude::*;

// we could calculate the number of nodes during runtime
// even better: fixed number at compile time
const NUMBER_NODES: usize = 10000;

#[derive(Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub(crate) source: u32,
    pub(crate) target: u32,
    distance: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Node {
    pub lat: f64,
    pub lon: f64,
}

impl Into<(f64,f64)> for Node {
    fn into(self) -> (f64, f64) {
        (self.lon, self.lat)
    }
}

pub struct GridGraph {
    pub number_nodes: i64,
    // index equals node id
    // defines number of neighbors for node at index
    pub offsets: Vec<u32>,
    pub edges: Vec<Edge>,
    // index equals node id
    pub nodes: Vec<Node>,
}

impl GridGraph {
    pub fn default() -> GridGraph {
        GridGraph {
            number_nodes: 0,
            offsets: Vec::new(),
            edges: Vec::new(),
            nodes: Vec::new()
        }
    }
    pub fn new(polygon_test: &PointInPolygonTest) -> GridGraph {
        // mapping from virtual nodes indices (0..NUMBER_NODES) (includes nodes inside of polygons) to the actual nodes of the grid (includes only nodes of the graph)
        let mut virtual_nodes_to_index: Vec<Option<u32>> = vec![None;NUMBER_NODES];
        let mut number_virtual_nodes: usize = 0;
        let mut number_graph_nodes: usize = 0;
        let mut nodes = vec![Node { lat: 0.0, lon: 0.0}; NUMBER_NODES];
        let mut edges: Vec<Vec<Edge>> = vec![Vec::with_capacity(8); NUMBER_NODES];

        // algorithm taken from here https://www.cmu.edu/biolphys/deserno/pdf/sphere_equi.pdf
        // number of nodes is only very close not equal to NUMBER_NODES
        let N = NUMBER_NODES as f64;
        let pi = PI as f64;
        let radius_earth: f64 = 1.0; // in km
        let a: f64 = 4.0 * pi * (radius_earth.powf(2.0) / N);
        let d: f64 = a.sqrt();
        let m_theta = (pi / d).round() as i32;
        let d_theta: f64 = pi / (m_theta as f64);
        let d_phi: f64 = a / d_theta;
        let mut m_phi = 0;
        let mut number_azimuth_steps_last_round = 0;
        let mut number_virtual_nodes_before_last_round = 0;
        // calculated in rad!!
        for m in 0..m_theta {
            if ((number_virtual_nodes as f64 / NUMBER_NODES as f64)*100.0).ceil() as i32 > ((number_virtual_nodes_before_last_round as f64 / NUMBER_NODES as f64)*100.0).ceil() as i32 {
                println!("Generating graph: {}%", ((number_virtual_nodes as f64 / NUMBER_NODES as f64) * 100.0).ceil() as i32);
            }
            let polar = pi * ((m as f64) + 0.5) / (m_theta as f64);
            m_phi = ((2.0 * pi * (polar).sin() / d_phi).round() as i32);
            let number_azimuth_steps_this_round = (0..m_phi).len();
            //println!("Breitengrad: {} Punkte: {}, Indices: {:?}", m, m_phi, number_placed_nodes..(number_placed_nodes+m_phi as usize));
            let number_virtual_nodes_at_start_of_this_round = number_virtual_nodes;
            // Do point in polygon test in parallel and collect results
            let nodes_to_place : Vec<(i32, Option<Node>)> = (0..m_phi).into_par_iter().map(|n| {
                let azimuthal = 2.0 * pi * (n as f64) / (m_phi as f64);

                // convert rad to degrees and lon = polar - 90; lat = azimuthal-180
                let lon = azimuthal * (180.0 / pi) - 180.0;
                let lat = polar * (180.0 / pi) - 90.0;

                if polygon_test.check_intersection(*&(lon, lat)) {
                    (n, None)
                } else {
                    let source_node = Node {lat, lon};
                    (n, Some(source_node))
                }
            }).collect();
            nodes_to_place.into_iter().for_each(|(n, source_node_option)| {
                // println!("n: {}", n);
                let n_float = n as f64;
                if let Some(source_node) = source_node_option {
                if number_virtual_nodes < NUMBER_NODES {
                    nodes[number_graph_nodes] = source_node;
                    virtual_nodes_to_index[number_virtual_nodes] = Some(number_graph_nodes as u32);
                    if number_azimuth_steps_last_round > 3 {
                        let last_round_factor = ((number_azimuth_steps_this_round as f64 - n_float) / (number_azimuth_steps_this_round as f64));

                        let offset_float = (n_float + (number_azimuth_steps_last_round as f64) * last_round_factor);

                        //let node_floor = number_placed_nodes - (offset_float.floor() + if last_round_factor < 1.0 {number_pref_azimuth_steps_last as f64} else {0.0}) as usize;
                        //let node_above = if calculate_length_between_points_on_sphere_with_radius_one(&source_node, &nodes[number_placed_nodes-offset_float.floor() as usize])
                        //    < calculate_length_between_points_on_sphere_with_radius_one(&source_node, &nodes[number_placed_nodes - offset_float.ceil() as usize]) {number_placed_nodes - offset_float.floor() as usize} else {number_placed_nodes - offset_float.ceil() as usize};
                        let virtual_index_top_right_node = number_virtual_nodes - offset_float.floor() as usize;
                        let virtual_index_top_left_node = number_virtual_nodes - offset_float.ceil() as usize;
                        // number_virtual_nodes_at_start_of_this_round + ((m_phi + n - 1) % m_phi) as usize;
                        let virtual_node_index_neighbor = calc_index_modulo(&number_virtual_nodes_at_start_of_this_round, &(m_phi as usize), number_virtual_nodes + (m_phi -1) as usize);
                        if n == m_phi - 1 && number_virtual_nodes_at_start_of_this_round > 1 {
                            // connect top manually
                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[number_virtual_nodes_at_start_of_this_round - 1]);
                        }
                        if virtual_index_top_left_node == virtual_index_top_right_node {
                            // node is exactly above -> also add edges to the nodes right and left
                            // Todo: handle wrap around if index is below first node of the last circle
                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node -1)]);
                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node +1)]);
                        } else {
                            // add edges to the two nearest nodes above

                            // Todo: handle wrap around if index is below first node of the last circle
                            let distance_right = add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node)]);
                            let distance_left = add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node)]);
                            // insert third node on the other side of the nearest node
                            match (distance_right, distance_left) {
                                (None, Some(_)) => {add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]);}
                                (Some(_), None) => {add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node + 1)]);}
                                (Some(dst_right), Some(dst_left)) => {
                                    if dst_left != dst_right {
                                        if dst_left > dst_right {
                                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_right_node + 1)]);
                                        } else {
                                            add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[calc_index_modulo(&number_virtual_nodes_before_last_round, &number_azimuth_steps_last_round, virtual_index_top_left_node - 1)]);
                                        }
                                    }
                                }
                                (_, _) => {}
                            }
                        }
                        // add edge to neighbor
                       add_edge(&mut edges, &nodes, number_graph_nodes, &virtual_nodes_to_index[virtual_node_index_neighbor]);
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
        let mut offsets = Vec::with_capacity(edges.len()+1);
        offsets.push(0);
        for i in 0..number_graph_nodes {
            let last_offset = offsets.last().unwrap();
            offsets.push(edges[i].len() as u32 + *last_offset);
        }
        let flattened_edges: Vec<Edge> = edges.concat();
        println!("number even distributed nodes {}", number_virtual_nodes);
        println!("number placed nodes {}", number_graph_nodes);
        println!("number edges {}", flattened_edges.len());

        GridGraph {
            number_nodes: number_graph_nodes as i64,
            edges: flattened_edges,
            offsets,
            nodes,
        }
    }
}

fn add_edge(edges: &mut Vec<Vec<Edge>>, nodes: &Vec<Node>, node1_idx: usize, node2_idx_option: &Option<u32>) -> Option<f64>{
    if let Some(node2_idx) = node2_idx_option {
        // target node is part of the graph
        let distance = calculate_length_between_points_on_sphere(&nodes[node1_idx as usize], &nodes[*node2_idx as usize]);
        edges[node1_idx].push(Edge{source: node1_idx as u32, target: *node2_idx, distance: distance as u32});
        edges[*node2_idx as usize].push(Edge{source: *node2_idx, target: node1_idx as u32, distance: distance as u32});
        return Some(distance);
    }
    return None;
}

fn calc_index_modulo(round_start_index: &usize, nodes_in_rounds: &usize, mut index_usize: usize) -> usize {
    let mut index = index_usize as isize;
    index = index - *round_start_index as isize;
    let new_index = (index % *nodes_in_rounds as isize) + *round_start_index as isize;
    if new_index > NUMBER_NODES as isize{
        println!("Calculated index would be out of bounds: start round index: {}, nodes in round: {}, index before: {}, index after modulo: {}", round_start_index, nodes_in_rounds, index_usize, new_index);
        return (new_index - *nodes_in_rounds as isize) as usize;
    }
    if new_index < 0 {
        (new_index + *nodes_in_rounds as isize) as usize
    } else {
        new_index as usize
    }

}

const EARTH_RADIUS: f64 = 6_378_137_f64; // earth radius in meters

fn calculate_length_between_points_on_sphere(node1: &Node, node2: &Node) -> f64 {
    distance(node1.lon, node1.lat, node2.lon, node2.lat)
}

// expects lat/lon in degrees
fn distance(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let lat1 = lat1 * ((PI / 180.0) as f64); // convert to rad
    let lat2 = lat2 * ((PI / 180.0) as f64);
    let lon1 = lon1 * ((PI / 180.0) as f64);
    let lon2 = lon2 * ((PI / 180.0) as f64);
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powf(2.0) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powf(2.0);
    let c = 2.0 * (a.sqrt()).asin();
    return EARTH_RADIUS * c;
}

fn to_lat_lon(theta: f64, phi: f64) -> (f64, f64) {
    let radius_earth = 6345 as f64; // radius earth in km
    // convert to cartesian first
    let x = radius_earth * theta.sin() * phi.cos();
    let y = radius_earth * theta.sin() * phi.sin();
    let z = radius_earth * theta.cos();

    // cartesian to wgs84 following the first answer
    // p = 90-lat
    // lat = 90-p
    // https://gis.stackexchange.com/questions/287467/translate-cartesian-coordinates-to-wgs84-preferably-in-pyproj
    let r = (x.powf(2.0) + y.powf(2.0)).sqrt() as f64;
    let lat = (180.0 * (r / radius_earth).acos() / (PI as f64));
    let lon = (180.0 * y.atan2(x) / (PI as f64));
    (lon, lat)
}
