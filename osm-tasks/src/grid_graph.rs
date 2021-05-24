/*
simple grid graph representation following a classic adjacency list

nodes are represented by ids, edges by tuples where the first element is the outgoing and the seconds element the receiving node
 */

use std::f32::consts::PI;
use std::f32;
use core::num::FpCategory::Nan;
use crate::polygon_test::PointInPolygonTest;

// we could calculate the number of nodes during runtime
// even better: fixed number at compile time
const NUMBER_NODES: usize = 10000;
const NUMBER_NEIGHBORS: i32 = 4;

#[derive(Clone, Copy)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    distance: i32,
}

#[derive(Clone, Copy)]
pub struct Node {
    pub lat: f64,
    pub lon: f64,
}

pub struct GridGraph {
    pub number_nodes: i64,
    // index equals node id
    // defines number of neighbors for node at index
    pub offsets: Vec<i64>,
    pub edges: Vec<Edge>,
    // index equals node id
    pub nodes: Vec<Node>,
}

impl GridGraph {
    pub fn new(polygon_test: PointInPolygonTest) -> GridGraph {
        let mut number_placed_nodes: usize = 0;
        let mut number_edges: usize = 0;
        let edge_offset = 3;
        let mut nodes = vec![Node { lat: 0.0, lon: 0.0 }; NUMBER_NODES];
        let mut edges: Vec<Edge> = Vec::new();
        let offsets = vec![edge_offset; NUMBER_NODES];

        // algorithm taken from here https://www.cmu.edu/biolphys/deserno/pdf/sphere_equi.pdf
        // number of nodes is only very close not equal to NUMBER_NODES
        let n = NUMBER_NODES as f64;
        let pi = PI as f64;
        let radius_earth: f64 = 1.0; // in km
        let a: f64 = 4.0 * pi * (radius_earth.powf(2.0) / n);
        let d: f64 = a.sqrt();
        let m_theta = (pi / d).round() as i32;
        let d_theta: f64 = pi / (m_theta as f64);
        let d_phi: f64 = a / d_theta;
        let mut m_phi = 0;

        // calculated in rad!!
        for m in 0..m_theta {
            let polar = pi * ((m as f64) + 0.5) / (m_theta as f64);
            m_phi = ((2.0 * pi * (polar).sin() / d_phi).round() as i32);
            for n in 0..m_phi {
                let azimuthal = 2.0 * pi * (n as f64) / (m_phi as f64);
                if number_placed_nodes < NUMBER_NODES {
                    // convert rad to degrees and lon = polar - 90; lat = azimuthal-180
                    let lat = azimuthal * (180.0 / pi) - 180.0;
                    let lon = polar * (180.0 / pi) - 90.0;
                    let ignore_node = polygon_test.check_intersection((lat,lon));
                    if !ignore_node {
                        let source_node = Node { lat, lon };
                        nodes[number_placed_nodes] = source_node;
                        number_placed_nodes += 1;
                    }
                }
            }
        }

        for n in 0..number_placed_nodes {
            let mut k = 0;

            // used for shifting during selection sort
            // maps real index to swapped nodes
            let mut nodes_tmp: Vec<usize> = (0..number_placed_nodes).map(|x| x).collect();

            // use selection sort to find k closest neighbors
            for i in 0..number_placed_nodes {
                let mut small = i;
                let mut curr_dis = distance_nodes(nodes[nodes_tmp[n]], nodes[nodes_tmp[i]]);
                for j in (i+1)..number_placed_nodes {
                    let dis = distance_nodes(nodes[nodes_tmp[n]], nodes[nodes_tmp[j]]);
                    if dis < curr_dis && dis != 0.0 {
                        small = j;
                        curr_dis = dis;
                    }
                }
                k += 1;
                let tmp = nodes_tmp[small];
                nodes_tmp[small] = nodes_tmp[i];
                nodes_tmp[i] = tmp;
                edges.push(Edge { source: n, target: small, distance: curr_dis as i32 });

                // we only need k neighbors
                if k > NUMBER_NEIGHBORS {
                    break;
                }
            }
        }

        println!("number nodes {}", number_placed_nodes);
        println!("number edges {}", edges.len());

        GridGraph {
            number_nodes: NUMBER_NODES as i64,
            edges,
            offsets,
            nodes,
        }
    }
}

fn distance_nodes(node1: Node, node2: Node) -> f64 {
    return distance(node1.lon, node2.lon, node1.lat, node2.lat);
}

// expects lat/lon in degrees
fn distance(lat1: f64, lat2: f64, lon1: f64, lon2: f64) -> f64 {
    let r = 6371.0; // rad earth in km
    let lat1 = lat1 * ((PI / 180.0) as f64); // convert to rad
    let lat2 = lat2 * ((PI / 180.0) as f64);
    let lon1 = lon1 * ((PI / 180.0) as f64);
    let lon2 = lon2 * ((PI / 180.0) as f64);
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powf(2.0) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powf(2.0);
    let c = 2.0 * (a.sqrt()).asin();
    return r * c;
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
