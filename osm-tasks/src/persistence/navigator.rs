use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};
use crate::model::benchmark::{BenchmarkResult, CollectedBenchmarks};

pub trait Navigator: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn build_graph(&mut self, number_nodes: usize);
    fn calculate_route(&mut self, route_request: RouteRequest) -> Option<ShipRoute>;
    fn benchmark_dijkstra(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult>;
    fn benchmark_a_star(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult>;
    fn benchmark_bd_dijkstra(&mut self, start_node: u32, end_node: u32, query_id: usize) -> Option<BenchmarkResult>;
    fn get_number_nodes(&self) -> u32;
    fn test_ch(&mut self);
    fn run_benchmarks(&mut self, nmb_queries: usize) -> CollectedBenchmarks;
}
