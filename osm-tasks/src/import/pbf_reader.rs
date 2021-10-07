use std::time::Instant;
use std::collections::{HashMap, HashSet};
use osmpbf::Element;
use rayon::prelude::*;
use crate::algorithms::polygon_test::PointInPolygonTest;
use osmpbf::ElementReader;
use std::fs::File;
use std::io::{Write, BufWriter, BufReader};
use core::iter;
use std::iter::FromIterator;
use rand::distributions::{Distribution, Uniform};
use std::slice::Iter;
use std::path::Path;
use std::ffi::OsStr;
use crate::export::json_generator::JsonBuilder;
use crate::export::kml_exporter::KmlExport;
use crate::model::grid_graph::GridGraph;
use crate::model::cn_model::CNMetadata;
use crate::algorithms::cn_graph_creator::CNGraphCreator;

/// tries to load the graph for this from disk and builds the graph if prebuild graph was found.
pub(crate) fn read_or_create_graph<S: AsRef<OsStr> + ?Sized>(osm_path_name: &S, force_create: bool, number_nodes: usize) -> GridGraph {
    let osm_path = Path::new(osm_path_name);
    let osm_name = osm_path.file_name().unwrap();
    let mut graph_file_name = osm_name.to_str().unwrap().to_owned();
    graph_file_name.push_str(".");
    graph_file_name.push_str(&*number_nodes.to_string());
    graph_file_name.push_str(".bin_new");
    println!("force create? {}, filename {}", force_create, graph_file_name);
    let path = osm_path.with_file_name(graph_file_name);
    if !force_create {
        let disk_graph = load_graph_from_disk(&path);
        if disk_graph.is_ok() {
            let gra = disk_graph.unwrap();
            println!("Loaded graph from disk \"{}\". Node count: {}", path.to_str().unwrap(), gra.nodes.len());
            return gra;
        } else {
            println!("graph not ok");
        }
    }
    let polygons = read_file(osm_path.to_str().unwrap());
    let polygon_test = PointInPolygonTest::new(polygons);

    // assign new value to the GRAPH reference
    let gra = GridGraph::new(&polygon_test, number_nodes);
    save_graph_to_disk(&path, &gra);
    println!("Saved graph to disk at {}", path.to_str().unwrap());

    return gra;
}

pub(crate) fn read_or_create_cn_metadata<S: AsRef<OsStr> + ?Sized>(osm_path_name: &S, force_recreate: bool, number_nodes: usize, initial_graph: &GridGraph) -> CNMetadata {
    let osm_path = Path::new(osm_path_name);
    let osm_name = osm_path.file_name().unwrap();
    let mut graph_file_name = osm_name.to_str().unwrap().to_owned();
    graph_file_name.push_str(".");
    graph_file_name.push_str(&*number_nodes.to_string());
    graph_file_name.push_str(".cn_meta");

    let path = osm_path.with_file_name(graph_file_name);

    println!("trying to load {}", path.to_str().expect("failed"));
    if !force_recreate {
        let disk_graph = load_cn_meta_from_disk(&path);
        if disk_graph.is_ok() {
            let gra = disk_graph.unwrap();
            println!("Loaded cn metadata from disk \"{}\". Shortcut count: {}", path.to_str().unwrap(), gra.get_shortcut.keys().len());
            return gra;
        } else {
            println!("cn metadata not ok");
        }
    }
    return create_save_cn_metadata(&path, initial_graph)
}

fn create_save_cn_metadata(path: &Path, initial_graph: &GridGraph) -> CNMetadata {
    let mut creator = CNGraphCreator::new(initial_graph);
    let data = creator.build_cn_graph();
    save_cn_metadata_to_disk(path, &data);
    println!("saved cn metadata at {}", path.to_str().unwrap());
    return data;
}

fn save_cn_metadata_to_disk(path: &Path, meta: &CNMetadata) {
    let mut f = BufWriter::new(File::create(path).unwrap());
    if let Err(e) = bincode::serialize_into(&mut f, meta) {
        println!("Could not save cn metadata to disk: {:?}", e);
    }
}

fn save_graph_to_disk(path: &Path, graph: &GridGraph) {
    let mut f = BufWriter::new(File::create(path).unwrap());
    if let Err(e) = bincode::serialize_into(&mut f, graph) {
        println!("Could not save graph to disk: {:?}", e);
    }
}

fn load_graph_from_disk(path: &Path) -> bincode::Result<GridGraph> {
    let mut f = BufReader::new(File::open(path)?);
    bincode::deserialize_from(&mut f)
}

fn load_cn_meta_from_disk(path: &Path) -> bincode::Result<CNMetadata> {
    let mut f = BufReader::new(File::open(path)?);
    bincode::deserialize_from(&mut f)
}

pub fn read_file(path: &str) -> Vec<Vec<(f64, f64)>> {
    let start_time = Instant::now();
    let reader = ElementReader::from_path(path).expect(&*format!("failed to read file {}", path));

    // key is the first node of the way; value is a tuple containing the last node and the whole way
    let mut coastlines: HashMap<i64, (i64, Vec<i64>)> = HashMap::new();
    let mut node_to_location: HashMap<i64, (f64, f64)> = HashMap::new();
    println!("Reading file {}", path);

    /*
     Assumptions:
     - each coastline way ends with a node which is contained in another coastline way
    */
    if let Err(e) = reader.for_each(|item| {
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
    }) {
        println!("Could not read coastlines file: {:?}", e);
    }
    println!("Reading done in {} sec", start_time.elapsed().as_secs());
    let merge_start_time = Instant::now();
    let mut polygons: Vec<Vec<(f64, f64)>> = merge_ways_to_polygons1(coastlines, node_to_location);

    println!("Merged coastlines to {} polygons in {} sec", polygons.len(), merge_start_time.elapsed().as_secs());
    check_polygons_closed(&polygons);

    // sort polygons by size so that we check the bigger before the smaller ones
    polygons.sort_by(|a, b| b.len().cmp(&a.len()));

    /*
    let file = "poly";
    JsonBuilder::new(String::from(file)).add_polygons(polygons).build();
    println!("Generated json");*/

    /*let point_test = PointInPolygonTest::new(vec![polygons[3].clone()]);
    let lon_min = -20.342559814453125;
    let lon_max = -20.20832061767578;
    let lat_min = 63.39413573718524;
    let lat_max = 63.45864118848073;

    let points_in_polygon = test_random_points_in_polygon(&point_test, 10000, (lon_min, lon_max, lat_min, lat_max)); */
    //write_to_file("island".parse().unwrap(), points_to_json(points_in_polygon));
    //let mut kml = KmlExport::init();
    //points_in_polygon.into_iter().for_each(|p| { kml.add_point(p, None) });
    //let graph = GridGraph::new();
    //graph.nodes.into_iter().foreach(|n| { kml.add_point(n, None) });
    //kml.write_file("kml.kml".parse().unwrap());
    return polygons;
}

pub fn read_file_and_export_geojson(osm_path: &str, geojson_path: &str) {
    let polygons = read_file(osm_path);
    let mut builder = JsonBuilder::new(geojson_path.parse().unwrap());
    builder.add_polygons(polygons);
    builder.build();
}

pub fn write_to_file(name: String, data: String) {
    let mut file = File::create(name).expect("Could not open file");
    file.write_all(data.as_ref()).expect("Could not write file");
}

pub fn points_to_json(points: Vec<(f64, f64)>) -> String {
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

pub fn lines_to_json(lines: Vec<((f64, f64), (f64, f64))>) -> String {
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

#[allow(dead_code)]
fn export_polygons_with_resolution(polygons: Vec<Vec<(f64, f64)>>, path: String, max_nodes_per_polygon: usize) {
    let mut kml = KmlExport::init();
    polygons.into_iter().map(|poly| {
        if poly.len() > max_nodes_per_polygon {
            let inverse_factor = poly.len() / max_nodes_per_polygon;
            let first = *poly.first().unwrap();
            return vec![first].into_iter().chain(poly.into_iter().step_by(inverse_factor)).collect();
        }
        poly
    }).for_each(|poly| {
        kml.add_polygon(poly, None);
    });
    kml.write_file(path);
}

#[allow(dead_code)]
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
    }).filter(|(lon, _): &(f64, f64)| { !lon.is_nan() }).collect()
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

#[allow(dead_code)]
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


