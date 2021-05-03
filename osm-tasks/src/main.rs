use std::collections::{HashMap, LinkedList};
use std::time::Instant;

use osmpbf::{Element, ElementReader, Node, Way};
use rayon::prelude::*;

fn main() {
    println!("Hello, world!");

    let mut coastlines_map = HashMap::new();
    let mut ways_to_nodes_map = HashMap::new();

    let mut number_merged_nodes = 0_u64;
    let mut merged_ways: LinkedList<Coastline> = LinkedList::new();

    let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./iceland-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./sa-coastlines.osm.pbf").expect("failed");
    //let reader = ElementReader::from_path("./planet-coastlines.osm.pbf").expect("failed");
    let mut ways = 0_u64;
    let mut coastlines = 0_u64;
    /*let ways = reader.par_map_reduce(
        |element| {
            match element {
                Element::Way(_) => 1,
                _ => 0,
            }
        },
        || 0_u64,      // Zero is the identity value for addition
        |a, b| a + b   // Sum the partial results
    ).expect("fail");*/
// Increment the counter by one for each way.
    reader.for_each(|element| {
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
        let mut current_coastline = Coastline { nodes: vec![], ways: list };
        let mut reached_unconnected_node_top = false;
        let mut reached_unconnected_node_bottom = false;
        let mut top_node_coastline = *first_node_top;
        let mut bottom_node_coastline = first_pair.node;
        let mut top_way_coastline = first_pair.way;
        let mut bottom_way_coastline = first_pair.way;
        while !reached_unconnected_node_bottom || !reached_unconnected_node_top {
            // merge until reached unconnected nodes at both ends
            if bottom_node_coastline == top_node_coastline {
                // polygon closed
                break;
            }
            if !reached_unconnected_node_top {
                if let Some(ways_for_node) = coastlines_map.get(&top_node_coastline) {
                    if ways_for_node.len() != 2 {
                        // reached top end of connected ways
                        coastlines_map.remove(&top_node_coastline);
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
                        coastlines_map.remove(&top_node_coastline);
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
                    coastlines_map.remove(&top_node_coastline);
                    top_node_coastline = new_top_node_coastline;
                } else {
                    // Should not happen
                    println!("No entry in coastline map for top node {} of way {}", top_node_coastline, top_way_coastline);
                    reached_unconnected_node_top = true;
                }
                // check if the current coastline is already a closed polygon
            }
            if bottom_node_coastline == top_node_coastline {
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
        if top_node_coastline == bottom_node_coastline {
            println!("Finished looped coastline from {} (of way {}) to {} (of way {}) out of {} ways", top_node_coastline, top_way_coastline, bottom_node_coastline, bottom_way_coastline, current_coastline.ways.len());
        } else {
            println!("Finished partly coastline from {} (of way {}) to {} (of way {}) out of {} ways", top_node_coastline, top_way_coastline, bottom_node_coastline, bottom_way_coastline, current_coastline.ways.len());
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
}

struct Coastline {
    nodes: Vec<i64>,
    ways: LinkedList<i64>,
}

struct WayNodePair {
    way: i64,
    node: i64,
}
