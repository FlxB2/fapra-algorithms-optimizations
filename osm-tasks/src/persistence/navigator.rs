use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use crate::benchmark::{AlgoBenchmark, BenchmarkResult, CollectedBenchmarks};

pub trait Navigator: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn build_graph(&mut self);
    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute>;
    fn benchmark_dijkstra(&mut self, route_request: RouteRequest, query_id: usize) -> Option<BenchmarkResult>;
    fn get_number_nodes(&self) -> u32;
    fn run_benchmarks(&mut self, nmb_queries: usize) -> CollectedBenchmarks;
}
