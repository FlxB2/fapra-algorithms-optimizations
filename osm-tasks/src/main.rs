use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::slice::Iter;
use std::time::Instant;

use osmpbf::{Element, ElementReader};

use crate::json_generator::JsonBuilder;
use crate::grid_graph::GridGraph;

mod json_generator;
mod grid_graph;

fn main() {
    let grid = GridGraph::new();
    JsonBuilder::new(String::from("testii")).add_points(grid.coords).build();
    //read_file("./monaco-latest.osm.pbf");
}

fn read_file(path: &str) {
    let start_time = Instant::now();
    let reader = ElementReader::from_path(path).expect(&*format!("failed to read file {}", path));

    // key is the first node of the way; value is a tuple containing the last node and the whole way
    let mut coastlines: HashMap<i64, (i64, Vec<i64>)> = HashMap::new();
    let mut node_to_location: HashMap<i64, (f64, f64)> = HashMap::new();
    println!("Reading file {}", path);

    /**
     Assumptions:
     - each coastline way ends with a node which is contained in another coastline way
    **/
    reader.for_each(|item| {
        match item {
            Element::Way(way) => {
                if let Some(_) = way.tags().find(|(k, v)| *k == "natural" && *v == "coastline") {
                    let first_node_id = way.refs().next().expect("way does not contain any nodes");
                    if let Some(last) = way.refs().last() {
                        coastlines.insert(first_node_id, (last, way.refs().collect()));
                    }
                }
            }
            Element::Node(node) => {
                node_to_location.insert(node.id(), (node.lon(), node.lat()));
            }
            Element::DenseNode(node) => {
                node_to_location.insert(node.id(), (node.lon(), node.lat()));
            }
            _ => {}
        }
    });
    println!("Reading done");

    let polygons: Vec<Vec<(f64, f64)>> = merge_ways_to_polygons1(coastlines, node_to_location);

    println!("Merged polygons coastlines to {} polygons in {} sec", polygons.len(), start_time.elapsed().as_secs());
    check_polygons_closed(&polygons);

    let file = "poly";
    JsonBuilder::new(String::from(file)).add_polygons(polygons).build();
    println!("Generated json");

    let point_test = PointInPolygonTest::new(vec![]);
    let point_to_test = (-19.168936046854252, 64.97414701038572);
    println!("Check point in polygons: ({}, {}) is in polygons: {}", point_to_test.0, point_to_test.1, point_test.check_intersection(point_to_test));
}

fn merge_ways_to_polygons1(coastlines: HashMap<i64, (i64, Vec<i64>)>, node_to_location: HashMap<i64, (f64, f64)>) -> Vec<Vec<(f64, f64)>> {
    let mut polygons: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut visited: HashMap<i64, bool> = HashMap::new();
    for key in coastlines.keys() {
        visited.insert(*key, false);
    }

    for key in coastlines.keys() {
        let mut start = key;
        let mut poly: Vec<(f64, f64)> = Vec::new();

        loop {
            if let Some(visit) = visited.get(start) {
                if *visit == true {
                    break;
                }
            }

            if let Some((end, way)) = coastlines.get(start) {
                // add way to polygon
                for node in way {
                    if let Some((lat, lon)) = node_to_location.get(node) {
                        poly.push((*lat, *lon));
                    } else {
                        print!("could not find node")
                    }
                }
                visited.insert(*start, true);
                start = end;
            } else {
                break;
            }
        }
        polygons.push(poly);
    }
    return polygons;
}

fn merge_ways_to_polygons2(coastlines: HashMap<i64, (i64, Vec<i64>)>, node_to_location: HashMap<i64, (f64, f64)>) -> Vec<Vec<(f64, f64)>> {
    let mut unprocessed_coastlines: HashSet<&i64> = HashSet::from_iter(coastlines.keys());
    let mut polygons: Vec<Vec<(f64, f64)>> = vec![];

    while !unprocessed_coastlines.is_empty() {
        let first_node = **unprocessed_coastlines.iter().next().expect("Coastline already processed");
        let (mut next_node, nodes) = coastlines.get(&first_node).expect("coastline not found in map");
        unprocessed_coastlines.remove(&first_node);

        let mut polygon: Vec<(f64, f64)> = Vec::with_capacity(nodes.len());
        append_coords_from_map_for_nodes(&node_to_location, &mut polygon, &mut nodes.iter());
        while next_node != first_node {
            unprocessed_coastlines.remove(&next_node);
            reserve_space_if_below_threshold(&mut polygon, 2000, 5000);
            if let Some((next_next_node, nodes)) = coastlines.get(&next_node) {
                append_coords_from_map_for_nodes(&node_to_location, &mut polygon, &mut nodes[1..].iter());
                next_node = *next_next_node;
            } else {
                println!("Could not find next node {}", next_node);
                break;
            }
        }
        polygon.shrink_to_fit();
        reserve_space_if_below_threshold(&mut polygons, 1, 5000);
        polygons.push(polygon);
    }
    polygons.shrink_to_fit();
    return polygons;
}

#[inline]
fn append_coords_from_map_for_nodes(node_to_location: &HashMap<i64, (f64, f64)>, polygon: &mut Vec<(f64, f64)>, nodes: &mut Iter<i64>) {
    nodes.for_each(|node_id| {
        if let Some(coord) = node_to_location.get(node_id) {
            polygon.push(*coord);
        } else {
            //Should not happen
            println!("Could not resolve coord for node: {}", node_id)
        }
    });
}

#[inline]
// use this function to check periodically if the capacity of a vector is below a limit
// to avoid expensive memory allocation at every insert operation
fn reserve_space_if_below_threshold<T>(vector: &mut Vec<T>, minimum_size: usize, reserved_size: usize) {
    if vector.capacity() < minimum_size {
        vector.reserve(reserved_size);
    }
}

fn check_polygons_closed(polygons: &Vec<Vec<(f64, f64)>>) -> bool {
    let polygon_count = polygons.len();
    let closed_polygons_count = polygons.iter().filter(|polygon| !polygon.is_empty() && polygon.first() == polygon.last()).count();
    println!("{} of {} polygons are closed", closed_polygons_count, polygon_count);
    polygon_count == closed_polygons_count
}

//let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
//let reader = ElementReader::from_path("./iceland-latest.osm.pbf").expect("failed");
//let reader = ElementReader::from_path("./iceland-coastlines.osm.pbf").expect("failed");
//let reader = ElementReader::from_path("./sa-coastlines.osm.pbf").expect("failed");
//let reader = ElementReader::from_path("./planet-coastlines.osm.pbf").expect("failed");


struct PointInPolygonTest {
    bounding_boxes: Vec<(f64, f64, f64, f64)>,
    polygons: Vec<Vec<(f64, f64)>>,
}

impl PointInPolygonTest {
    fn new(polygons: Vec<Vec<(f64, f64)>>) -> PointInPolygonTest {
        println!("Polygon test instance with {} polygons", polygons.len());
        let bounding_boxes: Vec<(f64, f64, f64, f64)> = polygons.iter().map(|polygon| PointInPolygonTest::calculate_bounding_box(polygon)).collect();
        return PointInPolygonTest { bounding_boxes, polygons };
    }

    fn check_point_between_edges(point_lon: &f64, (e1_lon, e1_lat): &(f64, f64), (e2_lon, e2_lat): &(f64, f64)) -> bool {
        let intersection_lat = e1_lat + ((e2_lat - e1_lat) / (e2_lon - e1_lon)) * (point_lon - e1_lon);
        f64::min(*e1_lat, *e2_lat) <= intersection_lat && intersection_lat <= f64::max(*e1_lat, *e2_lat)
    }

    fn calculate_bounding_box(polygon: &Vec<(f64, f64)>) -> (f64, f64, f64, f64) {
        let mut lon_min = 180_f64;
        let mut lon_max = -180_f64;
        let mut lat_min = 180_f64;
        let mut lat_max = -180_f64;
        for (lon, lat) in polygon {
            lon_min = f64::min(lon_min, *lon);
            lon_max = f64::max(lon_max, *lon);
            lat_min = f64::min(lat_min, *lat);
            lat_max = f64::max(lat_max, *lat);
        }
        println!("Bounding Box: ({},{}) to ({},{})", lon_min, lat_min, lon_max, lat_max);
        (lon_min, lon_max, lat_min, lat_max)
    }

    fn check_intersecting_bounding_boxes(&self, (lon, lat): (f64, f64)) -> Vec<usize> {
        let mut matching_polygons: Vec<usize> = Vec::with_capacity(self.polygons.len());
        self.bounding_boxes.iter().enumerate().for_each(|(idx, (lon_min, lon_max, lat_min, lat_max))| {
            if lon >= *lon_min && lon <= *lon_max && lat >= *lat_min && lat <= *lat_max {
                matching_polygons.push(idx);
                println!("Point ({},{}) is inside bounding box of polygon {}", lon, lat, idx);
            }
        });
        matching_polygons.shrink_to_fit();
        return matching_polygons;
    }

    fn check_point_in_polygons(&self, (point_lon, point_lat): (f64, f64), polygon_indices: Vec<usize>) -> bool {
        // TODO: implement test
        let mut intersection_count_even = true;
        for polygon_idx in polygon_indices {
            let polygon = &self.polygons[polygon_idx];
            for i in 0..self.polygons.len() - 1 {
                // Todo handle intersection with the nodes as special case
                if polygon[i].1 > point_lat && polygon[i + 1].1 > point_lat {
                    continue;
                }
                if PointInPolygonTest::check_point_between_edges(&point_lon, &polygon[i], &polygon[i + 1]) {
                    intersection_count_even = !intersection_count_even;
                    println!("Intersection")
                }
            }
            if !intersection_count_even {
                return true;
            }
        }
        return false;
    }

    fn check_intersection(&self, point: (f64, f64)) -> bool {
        // first get all intersecting bounding boxes
        let polygons_to_check = self.check_intersecting_bounding_boxes(point.clone());
        // check these polygons with point in polygon test
        self.check_point_in_polygons(point, polygons_to_check)
    }
}
