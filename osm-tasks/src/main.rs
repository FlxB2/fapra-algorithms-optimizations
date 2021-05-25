#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::{iter, thread, mem};
use std::iter::FromIterator;
use std::slice::Iter;
use std::time::Instant;

use osmpbf::{Element, ElementReader};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;
use rocket_okapi::{openapi, routes_with_openapi};
use schemars::JsonSchema;

use crate::json_generator::JsonBuilder;
use crate::grid_graph::GridGraph;
use crate::kml_exporter::KML_export;
use crate::polygon_test::PointInPolygonTest;
use crate::pbf_reader::read_file;
use std::borrow::Borrow;
use rocket::State;
use okapi::openapi3::Response;
use serde::{Deserialize, Serialize};
use rocket_contrib::json::Json;
use lazy_static::lazy_static;
use std::sync::Mutex;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

mod grid_graph;
mod json_generator;
mod dijkstra;
mod kml_exporter;
mod polygon_test;
mod pbf_reader;


#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Status {
    file_read: bool,
    polygons_constructed: bool,
    graph_constructed: bool,
    kml_generated: bool,
}

lazy_static! {
    static ref STATUS: Mutex<Status> = Mutex::new(
        Status { file_read: false, polygons_constructed: false, graph_constructed: false, kml_generated: false});
    static ref GRAPH: Mutex<GridGraph> = Mutex::new(GridGraph::default());
}

#[openapi]
#[get("/current_state")]
fn current_state() -> Json<Status> {
    Json(*STATUS.lock().unwrap())
}

fn main() {
    thread::spawn(|| {
        setup();
    });
    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes_with_openapi![current_state])
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}

fn setup() {
    //let polygons = read_file("./planet-coastlines.pbf.sec");
    let polygons = read_file("./iceland-coastlines.osm.pbf");
    STATUS.lock().unwrap().file_read = true;

    let polygon_test = PointInPolygonTest::new(polygons);
    STATUS.lock().unwrap().polygons_constructed = true;

    // assign new value to the GRAPH reference
    let mut global_graph = GRAPH.lock().expect("Could not lock graph");
    *global_graph = GridGraph::new(polygon_test);
    STATUS.lock().unwrap().graph_constructed = true;

    let mut kml = KML_export::init();
    let graph = global_graph;
    for n in 0..graph.edges.len() {
        let e = graph.edges[n];
        kml.add_linestring(Vec::from([
            (graph.nodes[e.source].lat, graph.nodes[e.source].lon),
            (graph.nodes[e.target].lat, graph.nodes[e.target].lon)]), Some(e.source.to_string()));
    }
    for n in 0..graph.nodes.len() {
        kml.add_point((graph.nodes[n].lat, graph.nodes[n].lon), None)
    }
    kml.write_file("kml.kml".parse().unwrap());
    STATUS.lock().unwrap().kml_generated = true;
}
