use crate::grid_graph::GridGraph;
use crate::persistence::in_memory_routing_repo::ShipRoute;

pub trait RoutingRepo: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn add_route(&mut self, route: ShipRoute) -> usize;
    fn get_route(&self, id: usize) -> Option<ShipRoute>;
    fn has_route(&self, id: usize) -> bool;
    fn get_job_id(&self) -> u32;
}
