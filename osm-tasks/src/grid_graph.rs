/*
simple grid graph representation following a classic adjacency list

nodes are represented by ids, edges by tuples where the first element is the outgoing and the seconds element the receiving node
 */

use std::f32::consts::PI;
use std::f32;

// we could calculate the number of nodes during runtime
// even better: fixed number at compile time
const NUMBER_NODES: usize = 1000;

pub struct GridGraph {
    pub number_nodes: i64,
    // allocating an array with 10^5+ values immediately leads to a stackoverflow
    // therefore we have to use vectors as far as i know
    // index equals node id
    pub neighbors: Vec<Vec<i64>>,
    // spherical coordinates 0: polar, 1: azimuthal; r is always radius of the earth
    pub coords: Vec<(f64, f64)>,
}

impl GridGraph {
    pub fn new() -> GridGraph {
        let mut neighbors = vec![vec![]; NUMBER_NODES];
        let mut coords = vec![(0.0, 0.0); NUMBER_NODES];

        let mut minmax: (f64, f64) = (432.0,0.0);

        // algorithm taken from here https://www.cmu.edu/biolphys/deserno/pdf/sphere_equi.pdf
        // number of nodes is only very close not equal to NUMBER_NODES
        let mut number_placed_nodes = 0;
        let n = NUMBER_NODES as f64;
        let pi = PI as f64;
        let radius_earth: f64 = 1.0; // in km
        let a: f64 = 4.0 * pi * (radius_earth.powf(2.0) / n);
        let d: f64 = a.sqrt();
        let m_theta = (pi / d).round() as i32;
        let d_theta: f64 = pi / (m_theta as f64);
        let d_phi: f64 = a / d_theta;

        // calculated in rad!!
        for m in 0..m_theta {
            let polar = pi * ((m as f64) + 0.5) / (m_theta as f64);
            let m_phi = ((2.0 * pi * (polar).sin() / d_phi).round() as i32);
            for n in 0..m_phi {
                let azimuthal = 2.0 * pi * (n as f64) / (m_phi as f64);
                if number_placed_nodes < NUMBER_NODES {
                    // convert rad to degrees and lat = polar - 90
                    coords[number_placed_nodes] = (azimuthal*(180.0/pi), polar*(180.0/pi)-90.0);
                    number_placed_nodes += 1;
                }

                if minmax.0 > azimuthal {
                    minmax.0 = azimuthal;
                }
                if minmax.1 < azimuthal {
                    minmax.1 = azimuthal;
                }
            }
        }
        println!("min {} max {}", minmax.0, minmax.1);

        println!("number nodes {}", number_placed_nodes);

        GridGraph {
            number_nodes: NUMBER_NODES as i64,
            neighbors,
            coords,
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
