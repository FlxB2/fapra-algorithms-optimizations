use std::collections::{HashMap, LinkedList};
use std::time::Instant;

use osmpbf::{Element, ElementReader};
use rayon::prelude::*;

fn main() {
    println!("Hello, world!");

    //let mut coastlines_map = HashMap::new();
    //let mut ways_to_nodes_map = HashMap::new();

    let mut number_merged_nodes = 0_u64;
    let mut merged_ways: LinkedList<Coastline> = LinkedList::new();

    //let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./sa-coastlines.osm.pbf").expect("failed");
    let reader = ElementReader::from_path("./planet-coastlines.osm.pbf").expect("failed");
    // let mut ways = 0_u64;
    let mut coastlines = 0_u64;
    let (coastlines_map_wrapped, ways_to_nodes_map_wrapped, node_coordinates_map_wrapped) = reader.par_map_reduce(
        |element| {
            if let Element::Way(way) = element {
                for (key, value) in way.tags() {
                    if let ("natural", "coastline") = (key, value) {
                        let nodes: Vec<i64> = way.refs().collect();
                        if nodes.len() <= 1 {
                            // Way with a single node does not give as any information -> discard
                            break;
                        }
                        if let Some(first_node) = nodes.first() {
                            if let Some(last_node) = nodes.last() {
                                if *first_node == *last_node {
                                    // already closed polygon
                                   /* let mut llist = LinkedList::new();
                                    llist.push_back(way.id());
                                    merged_ways.push_back(Coastline { nodes, ways: llist });
                                    number_merged_nodes += 1;*/
                                    // Todo: handle these ways
                                    println!("Discarded polygon. Fix this to handle this case correctly");
                                    break;
                                } else {
                                    return (MapOrEntry::Entries(vec![(*first_node, WayNodePair { way: way.id(), node: *last_node }),
                                                                     (*last_node, WayNodePair { way: way.id(), node: *first_node })]),
                                            MapOrEntry::Entries(vec![(way.id(), nodes )]),
                                    MapOrEntry::Entries(vec![]));
                                }
                            }
                        }
                        break;
                        // println!("key: {}, value: {}", key, value);
                    }
                }
            }else {
                if let Element::Node(node) = element {
                    return (MapOrEntry::Entries(vec![]), MapOrEntry::Entries(vec![]), MapOrEntry::Entries(vec![(node.id(),(node.lat(), node.lon()))]));
                }
            }
            (MapOrEntry::Entries(vec![]), MapOrEntry::Entries(vec![]), MapOrEntry::<(f64, f64)>::Entries(vec![]))
        },
        || (MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new()), MapOrEntry::Map(HashMap::new())),      // Zero is the identity value for addition
        |(a1,a2, a3),(b1, b2, b3)|{
            (MapOrEntry::<WayNodePair>::combine(a1, b1), MapOrEntry::<Vec<i64>>::combine(a2, b2), MapOrEntry::<(f64, f64)>::combine(a3, b3))
        }
    ).expect("fail");
    let mut coastlines_map = coastlines_map_wrapped.unwrap_map();
    let ways_to_nodes_map = ways_to_nodes_map_wrapped.unwrap_map();
    let node_coordinate_map = node_coordinates_map_wrapped.unwrap_map();
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
    coastlines_map.par_iter().for_each(|(k, v)| {
        if v.len() != 2 {
            println!("!Node is referenced by {} ways: {}", v.len(), k);
        }
    });

    let mut last_print_time = Instant::now();
    let start_time = Instant::now();
    while !coastlines_map.is_empty() {
        let (first_node_top, pair_list) = coastlines_map.iter().next().expect("Nodes to way map is empty");
        let first_pair = pair_list.first().expect("Pair list empty for node");
        let mut list = LinkedList::new();
        list.push_back(first_pair.way);
        let mut current_coastline = Coastline { nodes: vec![], ways: list, top_node: *first_node_top};
        let mut reached_unconnected_node_top = false;
        let mut reached_unconnected_node_bottom = false;
        let mut bottom_node_coastline = first_pair.node;
        let mut top_way_coastline = first_pair.way;
        let mut bottom_way_coastline = first_pair.way;
        while !reached_unconnected_node_bottom || !reached_unconnected_node_top {
            // merge until reached unconnected nodes at both ends
            if bottom_node_coastline == current_coastline.top_node {
                // polygon closed
                break;
            }
            if !reached_unconnected_node_top {
                if let Some(ways_for_node) = coastlines_map.get(&current_coastline.top_node) {
                    if ways_for_node.len() != 2 {
                        // reached top end of connected ways
                        coastlines_map.remove(&current_coastline.top_node);
                        reached_unconnected_node_top = true;
                        continue;
                    }
                    let other_way_node_pair = if ways_for_node.first().unwrap().way == top_way_coastline { ways_for_node.last().unwrap() } else { ways_for_node.first().unwrap() };
                    number_merged_nodes += 1;
                    if number_merged_nodes % 1000_u64 == 0_u64 {
                        let time_now = Instant::now();
                        println!("({}/{}, {}ms per 1000 merge) Merged nodes: {} {}", number_merged_nodes, coastlines, time_now.duration_since(last_print_time).as_millis(), top_way_coastline, other_way_node_pair.way);
                        last_print_time = time_now;
                    }
                    if top_way_coastline == other_way_node_pair.way {
                        // special case of the map data contains a way which is already a closed polygon
                        coastlines_map.remove(&current_coastline.top_node);
                        break;
                    }
                    current_coastline.ways.push_front(other_way_node_pair.way);
                    /*let nodes_of_other_way = ways_to_nodes_map.get(&other_way).expect("No nodes for other way");
                    // Check if we need to reverse the node list of the other way
                    if *nodes_of_other_way.last().unwrap() == top_node_coastline {
                        // other way nodes order matches the order of the coastline -> just join them
                        current_coastline.nodes = nodes_of_other_way.clone().into_iter().chain(current_coastline.nodes).collect();
                    } else {
                        // reverse nodes of other way before joining them
                        let mut reversed_other_nodes = nodes_of_other_way.clone();
                        reversed_other_nodes.reverse();
                        current_coastline.nodes = reversed_other_nodes.into_iter().chain(current_coastline.nodes).collect();
                    }*/
                    let new_top_node_coastline = other_way_node_pair.node;
                    top_way_coastline = other_way_node_pair.way;
                    coastlines_map.remove(&current_coastline.top_node);
                    current_coastline.top_node = new_top_node_coastline;
                } else {
                    // Should not happen
                    println!("No entry in coastline map for top node {} of way {}", current_coastline.top_node, top_way_coastline);
                    reached_unconnected_node_top = true;
                }
                // check if the current coastline is already a closed polygon
            }
            if bottom_node_coastline == current_coastline.top_node {
                // polygon closed
                break;
            }
            if !reached_unconnected_node_bottom {
                if let Some(ways_for_node) = coastlines_map.get(&bottom_node_coastline) {
                    if ways_for_node.len() != 2 {
                        // reached bottom end of connected ways
                        coastlines_map.remove(&bottom_node_coastline);
                        reached_unconnected_node_bottom = true;
                        continue;
                    }
                    let other_way_node_pair = if ways_for_node.first().unwrap().way == bottom_way_coastline { ways_for_node.last().unwrap() } else { ways_for_node.first().unwrap() };
                    number_merged_nodes += 1;
                    if number_merged_nodes % 1000_u64 == 0_u64 {
                        let time_now = Instant::now();
                        println!("({}/{}, {}ms per 1000 merge) Merged nodes: {} {}", number_merged_nodes, coastlines, time_now.duration_since(last_print_time).as_millis(), bottom_way_coastline, other_way_node_pair.way);
                        last_print_time = time_now;
                    }
                    assert_ne!(bottom_way_coastline, other_way_node_pair.way);
                    current_coastline.ways.push_back(other_way_node_pair.way);
                    /*
                    let nodes_of_other_way = ways_to_nodes_map.get(&other_way).expect("No nodes for other way");
                    // Check if we need to reverse the node list of the other way
                    if *nodes_of_other_way.first().unwrap() == bottom_node_coastline {
                        // other way nodes order matches the order of the coastline -> just join them
                        current_coastline.nodes.append(&mut nodes_of_other_way.clone());
                    } else {
                        // reverse nodes of other way before joining them
                        let mut reversed_other_nodes = nodes_of_other_way.clone();
                        reversed_other_nodes.reverse();
                        current_coastline.nodes.append(&mut reversed_other_nodes);
                    }
                    */
                    let new_bottom_node_coastline = other_way_node_pair.node;
                    bottom_way_coastline = other_way_node_pair.way;
                    coastlines_map.remove(&bottom_node_coastline);
                    bottom_node_coastline = new_bottom_node_coastline;
                } else {
                    // Should not happen
                    println!("No entry in coastline map for bottom node {} of way {}", bottom_node_coastline, bottom_way_coastline);
                    reached_unconnected_node_bottom = true;
                }
            }
        }
        if current_coastline.top_node == bottom_node_coastline {
            println!("Finished looped coastline from {} (of way {}) to {} (of way {}) out of {} ways", current_coastline.top_node, top_way_coastline, bottom_node_coastline, bottom_way_coastline, current_coastline.ways.len());
        } else {
            println!("Finished partly coastline from {} (of way {}) to {} (of way {}) out of {} ways", current_coastline.top_node, top_way_coastline, bottom_node_coastline, bottom_way_coastline, current_coastline.ways.len());
        }
        merged_ways.push_back(current_coastline);
    }
    println!("Merge took {} seconds", start_time.elapsed().as_secs());
    let mut number_one_way_coastlines = 0u64;
    merged_ways.iter().for_each(|e| {
        if e.ways.len() > 1 {
            println!("Coastline from way {} to {} merged out of {} ways", e.ways.front().unwrap(), e.ways.back().unwrap(), e.ways.len())
        }else {number_one_way_coastlines += 1;  }
    });
    println!("+ {} coastlines merged out of a single way", number_one_way_coastlines);


    merged_ways = merged_ways.into_iter().par_bridge().map(|mut coastline|{
        coastline.nodes.reserve(1000);
        let mut current_top_node = coastline.top_node;
        let mut coastline_nodes = coastline.nodes;
        coastline.ways.iter().for_each(|way|{
            let way_copy = *way;
            if let Some(nodes) = ways_to_nodes_map.get(&way_copy) {
                if !coastline_nodes.is_empty() {
                    // remove last node because the top node is part of the current nodes array and of
                    // the nodes array that is appended to it
                    coastline_nodes.remove(coastline_nodes.len() - 1);
                }
                if *nodes.first().unwrap().first().unwrap() == current_top_node {
                    // right order
                    coastline_nodes.append(&mut nodes.first().unwrap().clone());
                } else {
                    // nodes are reversed
                    let mut reversed_other_nodes =  nodes.first().unwrap().clone();
                    reversed_other_nodes.reverse();
                    coastline_nodes.append(&mut reversed_other_nodes);
                }
                current_top_node = *coastline_nodes.last().unwrap();
            }else{
                //Should not happen
                println!("Could not resolve node list for way: {}", way_copy)
            }
        });
        coastline.nodes = coastline_nodes;
        return coastline
    }).collect();

    let ways_to_coords_map : HashMap<i64, Vec<(f64, f64)>> = ways_to_nodes_map.into_iter().par_bridge().map(|(k,v)|{
        let mut coords : Vec<(f64,f64)> = Vec::with_capacity(v.len());
        v.iter().for_each(|nodeId|{
            if let Some(coord) = node_coordinate_map.get(nodeId.first().unwrap()){
                coords.push(*coord.first().unwrap());
            }
        });
        return (k, coords);
    }).collect();

    merged_ways.iter().par_bridge().for_each(|e|{
        println!("Coastline out of {} ways and {} nodes", e.ways.len(), e.nodes.len());
    })
}


struct Coastline {
    nodes: Vec<i64>,
    ways: LinkedList<i64>,
    top_node: i64
}

#[derive(Copy, Clone)]
struct WayNodePair {
    way: i64,
    node: i64
}

enum MapOrEntry<T>{
    Map(HashMap<i64, Vec<T>>),
    Entries(Vec<(i64, T)>)
}

    fn combineT<T>(a: MapOrEntry<T>, b: MapOrEntry<T>) -> MapOrEntry<T>{
        match (a, b) {
            (MapOrEntry::Map(mut map_a), MapOrEntry::Map(map_b))=>{
                map_a.extend(map_b);
                return MapOrEntry::Map(map_a);
            }
            (MapOrEntry::Map(mut map), MapOrEntry::Entries(entries)) | (MapOrEntry::Entries(entries), MapOrEntry::Map(mut map)) =>{
                entries.into_iter().for_each(|(key, entry)|{
                    if map.contains_key(&key) {
                        map.entry(key).and_modify(|e: &mut Vec<T>| { e.push(entry)});
                    }else {
                        map.insert(key,vec![entry]);
                    }
                });
                return MapOrEntry::Map(map);
            }
            (MapOrEntry::Entries(entries_a), MapOrEntry::Entries(entries_b)) => {
                let mut map = HashMap::new();
                entries_a.into_iter().for_each(|(key, entry)|{
                    if map.contains_key(&key){
                        map.entry(key).and_modify(|e: &mut Vec<T>| e.push(entry));
                    }else{
                        map.insert(key,vec![entry]);
                    }
                });
                entries_b.into_iter().for_each(|(key, entry)|{
                    if map.contains_key(&key){
                        map.entry(key).and_modify(|e: &mut Vec<T>| { e.push(entry)});
                    }else{
                        map.insert(key,vec![entry]);
                    }

                });
                return MapOrEntry::Map(map);
            }
        }
    }

impl <T> MapOrEntry<T>{
    fn unwrap_map(self) -> HashMap<i64, Vec<T>> {
        match self {
            MapOrEntry::Map(map) => map,
            _ => panic!("expected map"),
        }
    }

    fn combine<T1>(a: MapOrEntry<T1>, b: MapOrEntry<T1>) -> MapOrEntry<T1>{
        return combineT(a, b);
    }
}

