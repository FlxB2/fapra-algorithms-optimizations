use crate::persistence::in_memory_routing_repo::{ShipRoute, RouteRequest};

pub trait Navigator: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn build_graph(&mut self);
    fn calculate_route(&self, route_request: RouteRequest) -> Option<ShipRoute>;
    fn get_number_nodes(&self) -> u32;
}
