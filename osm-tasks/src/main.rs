use std::collections::{HashMap, LinkedList, HashSet};
use std::time::Instant;

use osmpbf::{Element, ElementReader};
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("Hello, world!");
    let main_start_time = Instant::now();

    //let mut coastlines_map = HashMap::new();
    //let mut ways_to_nodes_map = HashMap::new();

    let mut number_merged_nodes = 0_u64;
    let mut merged_ways: LinkedList<Coastline> = LinkedList::new();

    let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./sa-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./planet-coastlines.osm.pbf").expect("failed");
    // let mut ways = 0_u64;
    let mut coastlines = 0_u64;
    println!("Started file reading");
    let (coastlines_map_wrapped, first_node_to_nodes_map_wrapped, node_coordinates_map_wrapped) = reader.par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if let Some(_) = way.tags().find(|(k,v)| *k == "natural" && *v == "coastline"){
                            let nodes: Vec<i64> = way.refs().collect();
                            if nodes.len() <= 1 {
                                // Way with a single node does not give as any information -> discard
                                println!("Discarded way with only zero or one nodes: {}", way.id());
                            }else if let Some(first_node) = nodes.first() {
                                if let Some(last_node) = nodes.last() {
                                        return (MapOrEntry::Entry(*first_node,  *last_node),
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
        || (MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new())),      // Zero is the identity value for addition
        |(a1, a2, a3), (b1, b2, b3)| {
            (MapOrEntry::<i64>::combine(a1, b1), MapOrEntry::<Vec<i64>>::combine(a2, b2), MapOrEntry::<(f64, f64)>::combine(a3, b3))
        },
    ).expect("fail");
    let mut coastlines_map = coastlines_map_wrapped.unwrap_map();
    let mut first_node_to_nodes_map = first_node_to_nodes_map_wrapped.unwrap_map();
    let node_coordinate_map = node_coordinates_map_wrapped.unwrap_map();
    println!("Finished file reading");


// Increment the counter by one for each way.
    /*reader.for_each(|element| {
        if let Element::Way(way) = element {
            ways += 1;
            if ways % 1000_u64 == 0_u64 {
                println!("{}", ways / 1000_u64);
            }
            for (key, value) in way.tags() {
                if let ("natural", "coastline") = (key, value) {
                    coastlines += 1;
                    let nodes: Vec<_> = way.refs().collect();
                    if nodes.len() <= 1 {
                        // Way with a single node does not give as any information -> discard
                        break;
                    }
                    if let Some(first_node) = nodes.first() {
                        if let Some(last_node) = nodes.last() {
                            if *first_node == *last_node {
                                // already closed polygon
                                let mut llist = LinkedList::new();
                                llist.push_back(way.id());
                                merged_ways.push_back(Coastline { nodes, ways: llist });
                                number_merged_nodes += 1;
                                break;
                            } else {
                                coastlines_map.entry(*first_node)
                                    .and_modify(|e: &mut Vec<WayNodePair>| { e.push(WayNodePair { way: way.id(), node: *last_node }) })
                                    .or_insert(vec![WayNodePair { way: way.id(), node: *last_node }]);
                                coastlines_map.entry(*last_node)
                                    .and_modify(|e: &mut Vec<WayNodePair>| { e.push(WayNodePair { way: way.id(), node: *first_node }) })
                                    .or_insert(vec![WayNodePair { way: way.id(), node: *first_node }]);
                            }
                        }
                    }
                    ways_to_nodes_map.insert(way.id(), nodes);
                    break;
                    // println!("key: {}, value: {}", key, value);
                }
            }
        }
    }).expect("failed5");
        println!("{} of {} ways are coastlines", coastlines, ways);
     */

    println!("Number of ways {}", first_node_to_nodes_map.len());

    let mut last_print_time = Instant::now();
    let start_time = Instant::now();
    while !coastlines_map.is_empty() {
        let (first_node_ref, last_node) = coastlines_map.iter().next().expect("Nodes to way map is empty");
        let first_node = *first_node_ref;
        let mut list = LinkedList::new();
        let mut next_node = *last_node;
        coastlines_map.remove(&first_node);
        list.push_back(first_node);
        let mut current_coastline = Coastline { nodes: vec![], ways: list };

        while next_node != first_node {
            if let Some(next_next_node) = coastlines_map.remove(&next_node){
                current_coastline.ways.push_back(next_node);
                next_node = next_next_node;
            }else{
                println!("Could not find next node {}", next_node);
                break;
            }
        }
        if next_node == first_node {
            //println!("Finished looped coastline with start node {}", first_node);
        } else {
            println!("Finished partly coastline from {} to {} out of {} ways", first_node,next_node, current_coastline.ways.len());
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
        coastline_nodes.extend(first_node_to_nodes_map.get(&coastline.ways.pop_front().unwrap()).unwrap());
        coastline.ways.iter().for_each(|way| {
            if let Some(mut nodes) = first_node_to_nodes_map.get(way) {
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
    let mut closed_polygons: Vec<Vec<(f64, f64)>>= polygons.into_iter().par_bridge().filter(|e| e.len()>1 && e.first().unwrap() == e.last().unwrap()).collect();
    closed_polygons.sort_by(|a,b| b.len().cmp(&a.len()));
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

fn polygon_geojson_string(coords: &Vec<(f64, f64)>) -> String{
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
            (MapOrEntry::Map(mut map), MapOrEntry::Entry(k,v)) | (MapOrEntry::Entry(k,v), MapOrEntry::Map(mut map)) => {
                map.insert(k,v);
                MapOrEntry::Map(map)
            }
            (MapOrEntry::Entry(k1,v1), MapOrEntry::Entry(k2,v2)) => {
                let mut map = HashMap::new();
                map.insert(k1,v1);
                map.insert(k2,v2);
                MapOrEntry::Map(map)
            }
            (MapOrEntry::Map(mut map), MapOrEntry::NeutralElement()) | (MapOrEntry::NeutralElement(), MapOrEntry::Map(mut map)) => MapOrEntry::Map(map),
            (MapOrEntry::Entry(k,v), MapOrEntry::NeutralElement()) | (MapOrEntry::NeutralElement(), MapOrEntry::Entry(k, v)) => MapOrEntry::Entry(k, v),
            (MapOrEntry::NeutralElement(), MapOrEntry::NeutralElement()) => MapOrEntry::NeutralElement()
        }
    }
}

