/*
simple grid graph representation following a classic adjacency list

nodes are represented by ids, edges by tuples where the first element is the outgoing and the seconds element the receiving node
 */

use std::f32::consts::PI;
use std::f32;

// we could calculate the number of nodes during runtime
// even better: fixed number at compile time
const NUMBER_NODES: usize = 10000;

#[derive(Clone, Copy)]
pub struct Edge {
    pub(crate) source: Node,
    pub(crate) target: Node,
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
    pub fn new() -> GridGraph {
        let mut number_placed_nodes: usize = 0;
        let mut number_edges: usize = 0;
        let edge_offset = 3;
        let mut nodes = vec![Node { lat: 0.0, lon: 0.0 }; NUMBER_NODES];
        let mut edges = vec![Edge { source: Node { lat: 0.0, lon: 0.0 }, target: Node { lat: 0.0, lon: 0.0 }, distance: 0 }; NUMBER_NODES * 8];
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
            let number_pref_azimuth_steps = (0..m_phi).len();
            m_phi = ((2.0 * pi * (polar).sin() / d_phi).round() as i32);
            for n in 0..m_phi {
                let azimuthal = 2.0 * pi * (n as f64) / (m_phi as f64);
                if number_placed_nodes < NUMBER_NODES {
                    // convert rad to degrees and lon = polar - 90; lat = azimuthal-180
                    let lat = azimuthal * (180.0 / pi) - 180.0;
                    let lon = polar * (180.0 / pi) - 90.0;
                    let source_node = Node { lat, lon };
                    nodes[number_placed_nodes] = source_node;

                    if number_pref_azimuth_steps > 3 {
                        let indize_top_right = number_placed_nodes - number_pref_azimuth_steps;
                        let indize_top = (indize_top_right - 1);
                        let indize_top_left = (indize_top - 1);
                        let indize_left = number_placed_nodes - 1;
                        edges[number_edges] = Edge { source: source_node, target: nodes[indize_top_left], distance: 0 };
                        edges[number_edges + 1] = Edge { source: source_node, target: nodes[indize_top], distance: 0 };
                        edges[number_edges + 2] = Edge { source: source_node, target: nodes[indize_top_right], distance: 0 };
                        edges[number_edges + 3] = Edge { source: nodes[indize_top_left], target: source_node, distance: 0 };
                        edges[number_edges + 4] = Edge { source: nodes[indize_top], target: source_node, distance: 0 };
                        edges[number_edges + 5] = Edge { source: nodes[indize_top_right], target: source_node, distance: 0 };
                        edges[number_edges + 6] = Edge { source: source_node, target: nodes[indize_left], distance: 0 };
                        edges[number_edges + 7] = Edge { source: nodes[indize_left], target: source_node, distance: 0 };
                        number_edges += 7;
                    }
                    number_placed_nodes += 1;
                }
            }
        }

        println!("number nodes {}", number_placed_nodes);
        println!("number edges {}", number_edges);

        GridGraph {
            number_nodes: NUMBER_NODES as i64,
            edges,
            offsets,
            nodes,
        }
    }
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
