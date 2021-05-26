#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate rocket_contrib;

use std::{iter, mem, thread};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::ops::Deref;
use std::slice::Iter;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use osmpbf::{Element, ElementReader};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;
use rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, routes_with_openapi};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use serde::{Deserialize, Serialize};

use crate::grid_graph::{GridGraph, Node};
use crate::json_generator::JsonBuilder;
use crate::kml_exporter::KML_export;
use crate::navigator_use_case::NavigatorUseCase;
use crate::pbf_reader::read_file;
use crate::persistence::in_memory_navigator::InMemoryGraph;
use crate::persistence::in_memory_routing_repo::{InMemoryRoutingRepo, RouteRequest, ShipRoute};
use crate::persistence::navigator::Navigator;
use crate::persistence::routing_repo::RoutingRepo;
use crate::polygon_test::PointInPolygonTest;

mod grid_graph;
mod json_generator;
mod dijkstra;
mod kml_exporter;
mod polygon_test;
mod pbf_reader;
mod persistence;
mod navigator_use_case;

#[openapi]
#[post("/build_graph")]
fn build_graph(navigator_use_case: State<NavigatorUseCase>) {
    navigator_use_case.build_graph();
}

#[openapi]
#[get("/test_graph")]
fn test(navigator_use_case: State<NavigatorUseCase>) -> Json<u32> {
    Json(navigator_use_case.get_number_nodes())
}

// returns job id
#[openapi]
#[post("/route", format = "json", data = "<route_request>")]
fn route(route_request: Json<RouteRequest>, navigator_use_case: State<NavigatorUseCase>) -> Json<Option<u32>> {
    let id = navigator_use_case.calculate_route(route_request.0);
    Json(id)
}

// true if job is finished, false if not
#[openapi]
#[get("/jobStatus/<id>")]
fn job_status(id: usize, navigator_use_case: State<NavigatorUseCase>) -> Json<bool> {
    return Json(navigator_use_case.get_route(id).is_some());

}

#[openapi]
#[get("/jobResult/<id>")]
fn job_result(id: usize, navigator_use_case: State<NavigatorUseCase>) -> Option<Json<ShipRoute>> {
    let route = navigator_use_case.get_route(id);
    if route.is_some() {
        return Some(Json(route.unwrap()));
    }
    return None;
}

fn main() {
    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    let in_memory_routing_repo = InMemoryRoutingRepo::new();
    let routing_repo_mutex: Arc<Mutex<Box<dyn RoutingRepo>>> = Arc::new(Mutex::new(Box::new(in_memory_routing_repo)));
    let in_memory_navigator = InMemoryGraph::new();
    let navigator_mutex: Arc<Mutex<Box<dyn Navigator>>> = Arc::new(Mutex::new(Box::new(in_memory_navigator)));
    let navigator_use_case = NavigatorUseCase::new(Arc::clone(&navigator_mutex), Arc::clone(&routing_repo_mutex));
    rocket::ignite()
        .manage(navigator_use_case)
        .mount("/", routes_with_openapi![job_status, job_result, route, build_graph, test])
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}

fn setup() {
    /*
    let mut kml = KML_export::init();
    for n in 0..self.graph.edges.len() {
        let e = self.graph.edges[n];
        kml.add_linestring(Vec::from([
            (self.graph.nodes[e.source].lat, self.graph.nodes[e.source].lon),
            (self.graph.nodes[e.target].lat, self.graph.nodes[e.target].lon)]), Some(e.source.to_string()));
    }
    for n in 0..self.graph.nodes.len() {
        kml.add_point((self.graph.nodes[n].lat, self.graph.nodes[n].lon), None)
    }
    kml.write_file("kml.kml".parse().unwrap()); */
}
