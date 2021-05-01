use std::collections::HashMap;

use osmpbf::{Element, ElementReader, Node, Way};

fn main() {
    println!("Hello, world!");

    let mut coastlines_map = HashMap::new();
    let mut ways_to_nodes_map = HashMap::new();

    let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
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
                    if let Some(first_node) = nodes.first() {
                        coastlines_map.entry(*first_node)
                            .and_modify(|e: &mut Vec<i64>| { e.push(way.id()) })
                            .or_insert(vec![way.id()]);
                    }
                    if let Some(last_node) = nodes.last() {
                        coastlines_map.entry(*last_node)
                            .and_modify(|e: &mut Vec<i64>| { e.push(way.id()) })
                            .or_insert(vec![way.id()]);
                    }
                    ways_to_nodes_map.insert(way.id(), nodes);
                    break;
                    // println!("key: {}, value: {}", key, value);
                }
            }
        }
    }).expect("failed5");
    println!("{} of {} ways are coastlines", coastlines, ways);
    coastlines_map.iter().for_each(|(k, v)| {
        if v.len() != 2 {
            println!("!Node is referenced by {} ways: {}", v.len(), k);
        }
    });
    let mut number_merged_nodes = 0_u64;
    let mut merged_ways: Vec<Coastline> = vec![];
    while !coastlines_map.is_empty() {
        let first_way = *coastlines_map.iter().next().expect("Nodes to way map is empty").1.first().expect("Way list empty for node");
        let mut current_coastline = Coastline { nodes: ways_to_nodes_map.get(&first_way).expect("").clone(), ways: vec![first_way] };
        let mut reached_unconnected_node_top = false;
        let mut reached_unconnected_node_bottom = false;
        while !reached_unconnected_node_bottom || !reached_unconnected_node_top {
            // merge until reached unconnected nodes at both ends
            let top_node_coastline = *current_coastline.nodes.first().expect("Coastline has empty node list");
            let bottom_node_coastline = *current_coastline.nodes.last().expect("Coastline has empty node list");
            if !reached_unconnected_node_top {
                let top_way_coastline = current_coastline.ways.first().expect("Coastline has empty way list");
                if let Some(ways_for_node) = coastlines_map.get(&top_node_coastline) {
                    if ways_for_node.len() != 2 {
                        // reached top end of connected ways
                        coastlines_map.remove(&top_node_coastline);
                        reached_unconnected_node_top = true;
                        continue; // Todo: Bezieht sich das auf die richtige Schleife?
                    }
                    let other_way = *if ways_for_node.first().unwrap() == top_way_coastline { ways_for_node.last().unwrap() } else { ways_for_node.first().unwrap() };
                    number_merged_nodes += 1;
                    println!("({}/{}) Merged nodes: {} {}", number_merged_nodes, coastlines, top_way_coastline, other_way);
                    if *top_way_coastline == other_way {
                        // special case of the map data contains a way which is already a closed polygon
                        coastlines_map.remove(&top_node_coastline);
                        break;
                    }
                    current_coastline.ways.insert(0, other_way);
                    let nodes_of_other_way = ways_to_nodes_map.get(&other_way).expect("No nodes for other way");
                    // Check if we need to reverse the node list of the other way
                    if *nodes_of_other_way.last().unwrap() == top_node_coastline {
                        // other way nodes order matches the order of the coastline -> just join them
                        current_coastline.nodes = nodes_of_other_way.clone().into_iter().chain(current_coastline.nodes).collect();
                    } else {
                        // reverse nodes of other way before joining them
                        let mut reversed_other_nodes = nodes_of_other_way.clone();
                        reversed_other_nodes.reverse();
                        current_coastline.nodes = reversed_other_nodes.into_iter().chain(current_coastline.nodes).collect();
                    }
                    coastlines_map.remove(&top_node_coastline);
                } else {
                    // Should not happen
                    reached_unconnected_node_top = true;
                }
                // check if the current coastline is already a closed polygon
                if bottom_node_coastline == *current_coastline.nodes.first().unwrap() {
                    // polygon closed
                    break;
                }
            }
            if !reached_unconnected_node_bottom {
                let bottom_way_coastline = current_coastline.ways.last().expect("Coastline has empty way list");
                if let Some(ways_for_node) = coastlines_map.get(&bottom_node_coastline) {
                    if ways_for_node.len() != 2 {
                        // reached bottom end of connected ways
                        coastlines_map.remove(&bottom_node_coastline);
                        reached_unconnected_node_bottom = true;
                        continue; // Todo: Bezieht sich das auf die richtige Schleife?
                    }
                    let other_way = *if ways_for_node.first().unwrap() == bottom_way_coastline { ways_for_node.last().unwrap() } else { ways_for_node.first().unwrap() };
                    number_merged_nodes += 1;
                    println!("({}/{}) Merged nodes: {} {}", number_merged_nodes, coastlines, bottom_way_coastline, other_way);
                    assert_ne!(*bottom_way_coastline, other_way);
                    current_coastline.ways.push(other_way);
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
                    coastlines_map.remove(&bottom_node_coastline);
                } else {
                    // Should not happen
                    reached_unconnected_node_bottom = true;
                }
                // check if the current coastline is already a closed polygon
                if *current_coastline.nodes.first().unwrap() == *current_coastline.nodes.last().unwrap() {
                    // polygon closed
                    break;
                }
            }
        }
        merged_ways.push(current_coastline)
    }

    merged_ways.iter().for_each(|e| {
        println!("Coastline from {} to {} merged out of {} ways", e.nodes.first().unwrap(), e.nodes.last().unwrap(), e.ways.len())
    });
}

struct Coastline {
    nodes: Vec<i64>,
    ways: Vec<i64>,
}

struct WayNodePair<'a> {
    node: Node<'a>,
    way: Way<'a>,
}
