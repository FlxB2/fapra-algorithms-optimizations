#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
extern crate rocket_contrib;

use std::env;
use std::sync::{Arc, Mutex};

use rocket::State;
use rocket_contrib::json::Json;
use rocket_okapi::{openapi, routes_with_openapi};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use serde::{Deserialize, Serialize};

use model::benchmark::CollectedBenchmarks;

use crate::config::Config as LocalConfig;
use crate::cors::CORS;
use crate::model::grid_graph::Node;
use crate::navigator_use_case::NavigatorUseCase;
use crate::persistence::benchmark_repo::BenchmarkRepo;
use crate::persistence::in_memory_benchmark_repo::InMemoryBenchmarkRepo;
use crate::persistence::in_memory_navigator::InMemoryGraph;
use crate::persistence::in_memory_routing_repo::{InMemoryRoutingRepo, RouteRequest, ShipRoute};
use crate::persistence::navigator::Navigator;
use crate::persistence::routing_repo::RoutingRepo;
use crate::import::pbf_reader;
use rocket::config::{Config, Environment};

mod persistence;
mod navigator_use_case;
mod cors;
mod config;
mod export;
mod algorithms;
mod model;
mod import;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
struct Response {
    msg: String
}

#[openapi]
#[post("/buildGraph?<num_nodes>")]
fn build_graph(num_nodes: usize, navigator_use_case: State<NavigatorUseCase>) {
    navigator_use_case.build_graph(num_nodes);
}

#[openapi]
#[get("/testGraph")]
fn test(navigator_use_case: State<NavigatorUseCase>) -> Json<u32> {
    Json(navigator_use_case.get_number_nodes())
}

// returns job id
#[openapi]
#[get("/route?<lat_start>&<lon_start>&<lat_end>&<lon_end>")]
fn route(lat_start: f64, lon_start: f64, lat_end: f64, lon_end: f64, navigator_use_case: State<NavigatorUseCase>) -> Option<Json<Option<u32>>> {
    let route_request = RouteRequest {
        start: Node {
            lon: lon_start,
            lat: lat_start
        },
        end: Node {
            lon: lon_end,
            lat: lat_end
        }
    };
    let id = navigator_use_case.calculate_route(route_request);
    if id.is_some() {
        return Some(Json(id));
    }
    return None;
}

// true if job is finished, false if not
#[openapi]
#[get("/jobStatus?<id>")]
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

#[openapi]
#[post("/startBenchmark?<nmb_queries>")]
fn start_benchmark(nmb_queries: usize, navigator_use_case: State<NavigatorUseCase>) -> Json<Response> {
    navigator_use_case.benchmark(nmb_queries);
    Json(Response {
        msg: "started benchmark".parse().unwrap()
    })
}

#[openapi]
#[get("/isBenchmarkRunning")]
fn check_benchmark(navigator_use_case: State<NavigatorUseCase>) -> Json<bool> {
    if navigator_use_case.is_benchmark_finished() {
        return Json(true);
    }
    Json(false)
}

#[openapi]
#[get("/benchmarkResults")]
fn benchmark_results(navigator_use_case: State<NavigatorUseCase>) -> Option<Json<CollectedBenchmarks>> {
    let results = navigator_use_case.get_benchmark_results();
    if results.is_some() {
        return Some(Json(results.unwrap()));
    }
    None
}

#[openapi]
#[get("/testCH")]
fn test_ch(navigator_use_case: State<NavigatorUseCase>) {
    navigator_use_case.test_ch();
}

fn main() {
    LocalConfig::init();
    let config = LocalConfig::global();
    println!("Using file {} and a maximum number of {} nodes.", config.coastlines_file(), config.number_of_nodes());
    if let Some(geojson_path) = config.geojson_export_path().as_ref() {
        println!("Generate and export polygons as geoJSON");
        pbf_reader::read_file_and_export_geojson(config.coastlines_file(), geojson_path);
        println!("Generated geoJSON with polygons");
    }
    rocket()
}

fn rocket() {
    let in_memory_routing_repo = InMemoryRoutingRepo::new();
    let routing_repo_mutex: Arc<Mutex<Box<dyn RoutingRepo>>> = Arc::new(Mutex::new(Box::new(in_memory_routing_repo)));
    let in_memory_navigator = InMemoryGraph::new();
    let navigator_mutex: Arc<Mutex<Box<dyn Navigator>>> = Arc::new(Mutex::new(Box::new(in_memory_navigator)));
    let in_memory_benchmark_repo = InMemoryBenchmarkRepo::new();
    let benchmark_repo_mutex: Arc<Mutex<Box<dyn BenchmarkRepo>>> = Arc::new(Mutex::new(Box::new(in_memory_benchmark_repo)));
    let navigator_use_case = NavigatorUseCase::new(
        Arc::clone(&navigator_mutex), Arc::clone(&routing_repo_mutex), Arc::clone(&benchmark_repo_mutex));

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .finalize();

    rocket::custom(config.unwrap())
        .attach(CORS)
        .manage(navigator_use_case)
        .mount("/", routes_with_openapi![job_status, job_result, route, build_graph, test, start_benchmark, check_benchmark, benchmark_results, test_ch])
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .launch();
}
