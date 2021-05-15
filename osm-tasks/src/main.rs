use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::iter;
use std::iter::FromIterator;
use std::slice::Iter;
use std::time::Instant;

use osmpbf::{Element, ElementReader};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use crate::json_generator::JsonBuilder;

mod json_generator;

fn main() {
    //read_file("./monaco-latest.osm.pbf");
    read_file("./iceland-coastlines.osm.pbf");
    //read_file("./planet-coastlines.osm.pbf");
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
    println!("Reading done in {} sec", start_time.elapsed().as_secs());
    let merge_start_time = Instant::now();
    let mut polygons: Vec<Vec<(f64, f64)>> = merge_ways_to_polygons1(coastlines, node_to_location);

    println!("Merged polygons coastlines to {} polygons in {} sec", polygons.len(), merge_start_time.elapsed().as_secs());
    check_polygons_closed(&polygons);

    // sort polygons by size so that we check the bigger before the smaller ones
    polygons.sort_by(|a, b| b.len().cmp(&a.len()));
    println!("Number of Polygons: {}", polygons.first().unwrap().len());

    /*
    let file = "poly";
    JsonBuilder::new(String::from(file)).add_polygons(polygons).build();
    println!("Generated json");*/

    let point_test = PointInPolygonTest::new(vec![polygons[3].clone()]);
    //let point_to_test = ( -20.30324399471283, 63.430207573053615);
    //println!("Check point in polygons: ({}, {}) is in polygons: {}", point_to_test.0, point_to_test.1, point_test.check_intersection(point_to_test.clone()));

    //println!("idx {:?}", point_test.check_intersecting_bounding_boxes(point_to_test));
    let lon_min = -20.342559814453125;
    let lon_max = -20.20832061767578;
    let lat_min = 63.39413573718524;
    let lat_max = 63.45864118848073;

    let points_in_polygon = test_random_points_in_polygon(&point_test, 10000, (lon_min, lon_max, lat_min, lat_max));
    write_to_file("island".parse().unwrap(), points_to_json(points_in_polygon));
}

fn write_to_file(name: String, data: String) {
    let mut file = File::create(name).expect("Could not open file");
    file.write_all(data.as_ref()).expect("Could not write file");
}

fn points_to_json(points: Vec<(f64, f64)>) -> String {
    let points_string = format!("{:?}", points).replace("(", "[").replace(")", "]\n");
    let feature = format!("{{ \"type\": \"MultiPoint\",
    \"coordinates\": {}
}}", points_string);
    format!("{{
  \"type\": \"FeatureCollection\",
  \"features\": [
    {{
      \"type\": \"Feature\",
      \"properties\": {{}},
      \"geometry\":  {} \
    }}
  ]
}}", feature)
}

fn lines_to_json(lines: Vec<((f64, f64), (f64, f64))>) -> String {
    let mut features = String::new();
    for line in lines {
        let geometry = format!("{{ \"type\": \"LineString\",
    \"coordinates\": [[{},{}],[{},{}]]
}}", line.0.0, line.0.1, line.1.0, line.1.1);
        let feature = format!("{{
      \"type\": \"Feature\",
      \"properties\": {{}},
      \"geometry\":  {} \
    }}\n,", geometry);
        features = features + &*feature;
    }
    features.pop();
    format!("{{
  \"type\": \"FeatureCollection\",
  \"features\": [
    {}
  ]
}}", features)
}

fn test_random_points_in_polygon(polygon_test: &PointInPolygonTest, number_of_points_to_test: usize, (lon_min, lon_max, lat_min, lat_max): (f64, f64, f64, f64)) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    let rng_lat = Uniform::from(lat_min..lat_max);
    let rng_lon = Uniform::from(lon_min..lon_max);
    let coords: Vec<(f64, f64)> = iter::repeat(0).take(number_of_points_to_test).map(|_| {
        (rng_lon.sample(&mut rng), rng_lat.sample(&mut rng))
    }).collect();
    coords.into_par_iter().map(|test_point: (f64, f64)| {
        if polygon_test.check_intersection(test_point.clone()) {
            return test_point;
        }
        return (f64::NAN, f64::NAN);
    }).filter(|(lon, lat): &(f64, f64)| { !lon.is_nan() }).collect()
}

fn merge_ways_to_polygons1(coastlines: HashMap<i64, (i64, Vec<i64>)>, node_to_location: HashMap<i64, (f64, f64)>) -> Vec<Vec<(f64, f64)>> {
    let mut polygons: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut visited: HashMap<i64, bool> = HashMap::new();
    for key in coastlines.keys() {
        visited.insert(*key, false);
    }

    for key in coastlines.keys() {
        if *visited.get(key).unwrap_or(&true) {
            continue;
        }
        let mut start = key;
        let mut poly: Vec<(f64, f64)> = vec![*node_to_location.get(start).expect("Could not find coords for start node")];

        loop {
            if let Some((end, way)) = coastlines.get(start) {
                // add way to polygon
                for node in way[1..].iter() {
                    if let Some((lat, lon)) = node_to_location.get(node) {
                        poly.push((*lat, *lon));
                    } else {
                        print!("could not find coords for node {}", node)
                    }
                }
                visited.insert(*start, true);
                start = end;
            } else {
                println!("Could not find node {} in coastlines map", start);
                break;
            }
            if let Some(visit) = visited.get(start) {
                if *visit == true {
                    polygons.push(poly);
                    break;
                }
            } else {
                println!("Could not find node {} in visited map", start);
            }
        }
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


struct PointInPolygonTest {
    bounding_boxes: Vec<(f64, f64, f64, f64)>,
    polygons: Vec<Vec<(f64, f64)>>,
}

struct Point(f64, f64);

impl Point {
    fn new(lon: f64, lat: f64) -> Point {
        Point(lat, lon)
    }
    fn from((lon, lat): &(f64, f64)) -> Point {
        Point(*lat, *lon)
    }

    fn lat(&self) -> f64 { self.0 }
    fn lon(&self) -> f64 { self.1 }
}

impl PointInPolygonTest {
    fn new(polygons: Vec<Vec<(f64, f64)>>) -> PointInPolygonTest {
        // println!("Polygon test instance with {} polygons", polygons.len());
        let bounding_boxes: Vec<(f64, f64, f64, f64)> = polygons.iter().map(|polygon| PointInPolygonTest::calculate_bounding_box(polygon)).collect();
        return PointInPolygonTest { bounding_boxes, polygons };
    }

    fn check_point_between_edges((point_lon, point_lat): &(f64, f64), (v1_lon, v1_lat): &(f64, f64), (v2_lon, v2_lat): &(f64, f64)) -> bool {
        if v1_lon == v2_lon {
            // Ignore north-south edges
            return false;
        } else if v1_lat == v2_lat {
            return f64::min(*v1_lon, *v2_lon) <= *point_lon && *point_lon <= f64::max(*v1_lon, *v2_lon);
        } else if *point_lon < f64::min(*v1_lon, *v2_lon) || f64::max(*v1_lon, *v2_lon) < *point_lon {
            // Can not intersect with the edge
            return false;
        }
        // Todo: If both ends of the edge are in the northern hemisphere and the test point is south of the chord (on a lat-Ion projection) between the end points, it intersects the edge.

        let v1_lon_rad = v1_lon.to_radians();
        let v1_lat_tan = v1_lat.to_radians().tan();
        let v2_lon_rad = v2_lon.to_radians();
        let v2_lat_tan = v2_lat.to_radians().tan();
        let delta_v_lon_sin = (v1_lon_rad - v2_lon_rad).sin();
        let point_lon_rad = point_lon.to_radians();

        let intersection_lat_tan = (v1_lat_tan * ((point_lon_rad - v2_lon_rad).sin() / delta_v_lon_sin) - v2_lat_tan * ((point_lon_rad - v1_lon_rad).sin() / delta_v_lon_sin));
        if intersection_lat_tan == v1_lat_tan || intersection_lat_tan == v2_lat_tan {
            //special case: intersection is on one of the vertices
            let (hit_vert_lon_rad, other_vert_lon_rad) = if intersection_lat_tan == v1_lat_tan {(v1_lon_rad, v2_lon_rad)} else {(v2_lon_rad, v1_lon_rad)};
            // tread it as in polygon iff the other vertex is westward of the hit vertex
            return (hit_vert_lon_rad-other_vert_lon_rad).sin() > 0f64;
        }

        // intersection must be between the vertices and not below the point
        f64::min(v1_lat_tan, v2_lat_tan) <= intersection_lat_tan
            && intersection_lat_tan <= f64::max(v1_lat_tan, v2_lat_tan)
            && intersection_lat_tan >= point_lat.to_radians().tan()
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
        //println!("Bounding Box: ({},{}) to ({},{})", lon_min, lat_min, lon_max, lat_max);
        (lon_min, lon_max, lat_min, lat_max)
    }

    fn check_intersecting_bounding_boxes(&self, (lon, lat): (f64, f64)) -> Vec<usize> {
        let mut matching_polygons: Vec<usize> = Vec::with_capacity(self.polygons.len());
        self.bounding_boxes.iter().enumerate().for_each(|(idx, (lon_min, lon_max, lat_min, lat_max))| {
            if lon >= *lon_min && lon <= *lon_max && lat >= *lat_min && lat <= *lat_max {
                matching_polygons.push(idx);
                //println!("Point ({},{}) is inside bounding box of polygon {}", lon, lat, idx);
            }
        });
        matching_polygons.shrink_to_fit();
        return matching_polygons;
    }

    fn check_point_in_polygons(&self, (point_lon, point_lat): (f64, f64), polygon_indices: Vec<usize>) -> bool {
        let mut intersection_count_even = true;
        //let mut intersections: Vec<((f64, f64), (f64, f64))> = vec![];
        for polygon_idx in polygon_indices {
            intersection_count_even = true;
            let polygon = &self.polygons[polygon_idx];
            for i in 0..polygon.len() - 1 {
                if polygon[i].1 < point_lat && polygon[i + 1].1 < point_lat {
                    continue;
                }
                if polygon[i] == (point_lon, point_lat) {
                    // Point is at the vertex -> we define this as within the polygon
                    return true;
                }
                if PointInPolygonTest::check_point_between_edges(&(point_lon, point_lat), &polygon[i], &polygon[i + 1]) {
                    intersection_count_even = !intersection_count_even;
                    //  intersections.push((polygon[i], polygon[i + 1]));
                }
            }
            if !intersection_count_even {
                break;
            }
        }
        //write_to_file("lines".parse().unwrap(), lines_to_json(intersections));
        return !intersection_count_even;
    }
    const EARTH_RADIUS: i32 = 6_378_137;

    fn calculate_length_between_points(p1: &Point, p2: &Point) -> f64 {
        PointInPolygonTest::EARTH_RADIUS as f64 * ((p2.lon() - p1.lon()).powi(2) * ((p1.lat() + p2.lat()) / 2f64).cos().powi(2) * (p2.lat() - p1.lat()).powi(2)).sqrt()
    }

    fn check_intersection(&self, point: (f64, f64)) -> bool {
        // first get all intersecting bounding boxes
        let polygons_to_check = self.check_intersecting_bounding_boxes(point.clone());
        // check these polygons with point in polygon test
        self.check_point_in_polygons(point, polygons_to_check)
    }
}