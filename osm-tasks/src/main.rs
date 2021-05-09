mod json_generator;

use std::collections::{HashMap, LinkedList};
use std::time::Instant;

use osmpbf::{Element, ElementReader, Node, Way};
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use crate::json_generator::{JsonBuilder};

fn main() {
    read_file("./monaco-latest.osm.pbf");
}

fn read_file(mut path: &str) {
    let start_time = Instant::now();
    let reader = ElementReader::from_path(path).expect(&*format!("failed to read file {}", path));

    // key is the first node of the way; value is a tuple containing the last node and the whole way
    let mut coastlines: HashMap<i64, (i64, Vec<i64>)> = HashMap::new();
    let mut node_to_location: HashMap<i64, (f64, f64)> = HashMap::new();
    let mut polygons: Vec<Vec<(f64, f64)>> = Vec::new();
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


    let file = "poly";
    JsonBuilder::new(String::from(file)).add_polygons(polygons).build();
    println!("Generated json");
}

fn old() {
    let main_start_time = Instant::now();

    let mut number_merged_nodes = 0_u64;

    let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./sa-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./planet-coastlines.osm.pbf").expect("failed");

    println!("Started file reading");
    let (coastlines, first_node_to_way, node_to_coordinates) = reader.par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if let Some(_) = way.tags().find(|(k, v)| *k == "natural" && *v == "coastline") {
                        let nodes: Vec<i64> = way.refs().collect();
                        if nodes.len() <= 1 {
                            // Way with a single node does not give as any information -> discard
                            println!("Discarded way with only zero or one nodes: {}", way.id());
                        } else if let Some(first_node) = nodes.first() {
                            if let Some(last_node) = nodes.last() {
                                return (MapOrEntry::Entry(*first_node, *last_node),
                                        MapOrEntry::Entry(*first_node, nodes),
                                        MapOrEntry::NeutralElement());
                            }
                        }
                    }
                }
                Element::Node(node) => return (MapOrEntry::NeutralElement(), MapOrEntry::NeutralElement(), MapOrEntry::Entry(node.id(), (node.lon(), node.lat()))),
                Element::DenseNode(node) => return (MapOrEntry::NeutralElement(), MapOrEntry::NeutralElement(), MapOrEntry::Entry(node.id(), (node.lon(), node.lat()))),
                _ => {}
            }

            (MapOrEntry::NeutralElement(), MapOrEntry::<Vec<i64>>::NeutralElement(), MapOrEntry::<(f64, f64)>::NeutralElement())
        },
        || (MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new())),
        |(a1, a2, a3), (b1, b2, b3)| {
            (MapOrEntry::<i64>::combine(a1, b1), MapOrEntry::<Vec<i64>>::combine(a2, b2), MapOrEntry::<(f64, f64)>::combine(a3, b3))
        },
    ).expect("fail");

    // stores a map of coastlines, reference to first and last node
    let mut coastlines_map = coastlines.unwrap_map();
    // stores a map, mapping first node to way - shadow variable name
    let mut first_node_to_way = first_node_to_way.unwrap_map();
    // maps nodes to coordinates
    let node_coordinate_map = node_to_coordinates.unwrap_map();
    println!("Finished file reading");


    println!("Number of ways {}", first_node_to_way.len());

    let mut last_print_time = Instant::now();
    let start_time = Instant::now();
    let mut merged_ways: LinkedList<Coastline> = LinkedList::new();

    while !coastlines_map.is_empty() {
        let (first_node_ref, last_node) = coastlines_map.iter().next().expect("Nodes to way map is empty");
        let first_node = *first_node_ref;
        let mut next_node = *last_node;
        let mut list = LinkedList::new();
        coastlines_map.remove(&first_node);
        list.push_back(first_node);
        let mut current_coastline = Coastline { nodes: vec![], ways: list };

        while next_node != first_node {
            if let Some(next_next_node) = coastlines_map.remove(&next_node) {
                current_coastline.ways.push_back(next_node);
                next_node = next_next_node;
            } else {
                println!("Could not find next node {}", next_node);
                break;
            }
        }
        if next_node == first_node {
            //println!("Finished looped coastline with start node {}", first_node);
        } else {
            println!("Finished partly coastline from {} to {} out of {} ways", first_node, next_node, current_coastline.ways.len());
        }
        merged_ways.push_back(current_coastline);
    }
    println!("Merge took {} seconds", start_time.elapsed().as_secs());
    let mut number_one_way_coastlines = 0u64;

    merged_ways.iter().for_each(|e| {
        if e.ways.len() > 1 {
            println!("Coastline from way {} to {} merged out of {} ways", e.ways.front().unwrap(), e.ways.back().unwrap(), e.ways.len())
        } else { number_one_way_coastlines += 1; }
    });
    println!("+ {} coastlines merged out of a single way", number_one_way_coastlines);


    let polygons: Vec<Vec<(f64, f64)>> = merged_ways.into_iter().par_bridge().map(|mut coastline| {
        let mut coastline_nodes = coastline.nodes;
        coastline_nodes.extend(first_node_to_way.get(&coastline.ways.pop_front().unwrap()).unwrap());
        coastline.ways.iter().for_each(|way| {
            if let Some(mut nodes) = first_node_to_way.get(way) {
                coastline_nodes.extend(nodes[1..].iter());
            } else {
                //Should not happen
                println!("Could not resolve node list for way: {}", way)
            }
        });
        coastline.nodes = coastline_nodes;
        let mut coords: Vec<(f64, f64)> = Vec::with_capacity(coastline.nodes.len());
        coastline.nodes.iter().for_each(|node_id| {
            if let Some(coord) = node_coordinate_map.get(node_id) {
                coords.push(*coord);
            }
        });
        return coords;
    }).collect();

    let count_all_polygons = polygons.len();

    // filter for closed polygons since the others are more a line than a real polygon
    let mut closed_polygons: Vec<Vec<(f64, f64)>> = polygons;
    closed_polygons.sort_by(|a, b| b.len().cmp(&a.len()));
    closed_polygons.iter().take(15).par_bridge().for_each(|e| {
        println!("Polygon out of {} coords", e.len());
    });
    closed_polygons.iter().take(10).zip(1..10).par_bridge().for_each(|(coords, idx)| {
        let geojson_string = polygon_geojson_string(coords);
        let mut file = File::create(format!("polygon_{}.json", idx)).expect("could not open file");
        file.write_all(geojson_string.as_bytes()).expect("could not write to file");
    });

    println!("{} out of {} polygons are closed", closed_polygons.len(), count_all_polygons);

    println!("total time: {} sec", main_start_time.elapsed().as_secs());
}

fn polygon_geojson_string(coords: &Vec<(f64, f64)>) -> String {
    let coords_string = format!("{:?}", coords).replace("(", "[").replace(")", "]");
    format!("{{
      \"type\": \"Feature\",
      \"properties\": {{}},
      \"geometry\": {{
        \"type\": \"Polygon\",
        \"coordinates\": [
            {}
        ]
      }}
     }}", coords_string)
}

#[derive(Copy, Clone)]
struct WayNodePair {
    way: i64,
    node: i64,
}

struct Coastline {
    nodes: Vec<i64>,
    ways: LinkedList<i64>,
}

enum MapOrEntry<T> {
    Map(HashMap<i64, T>),
    Entry(i64, T),
    NeutralElement(),
}

impl<T> MapOrEntry<T> {
    fn unwrap_map(self) -> HashMap<i64, T> {
        match self {
            MapOrEntry::Map(map) => map,
            _ => panic!("expected map"),
        }
    }

    fn combine<T1>(a: MapOrEntry<T1>, b: MapOrEntry<T1>) -> MapOrEntry<T1> {
        return match (a, b) {
            (MapOrEntry::Map(mut map_a), MapOrEntry::Map(map_b)) => {
                map_a.extend(map_b);
                MapOrEntry::Map(map_a)
            }
            (MapOrEntry::Map(mut map), MapOrEntry::Entry(k, v)) | (MapOrEntry::Entry(k, v), MapOrEntry::Map(mut map)) => {
                map.insert(k, v);
                MapOrEntry::Map(map)
            }
            (MapOrEntry::Entry(k1, v1), MapOrEntry::Entry(k2, v2)) => {
                let mut map = HashMap::new();
                map.insert(k1, v1);
                map.insert(k2, v2);
                MapOrEntry::Map(map)
            }
            (MapOrEntry::Map(mut map), MapOrEntry::NeutralElement()) | (MapOrEntry::NeutralElement(), MapOrEntry::Map(mut map)) => MapOrEntry::Map(map),
            (MapOrEntry::Entry(k, v), MapOrEntry::NeutralElement()) | (MapOrEntry::NeutralElement(), MapOrEntry::Entry(k, v)) => MapOrEntry::Entry(k, v),
            (MapOrEntry::NeutralElement(), MapOrEntry::NeutralElement()) => MapOrEntry::NeutralElement()
        };
    }
}

