use crate::persistence::navigator::Navigator;
use std::sync::{Mutex, Arc};
use crate::persistence::in_memory_routing_repo::{RouteRequest, ShipRoute};
use std::thread;
use crate::persistence::routing_repo::RoutingRepo;

pub struct NavigatorUseCase {
    pub navigator: Arc<Mutex<Box<dyn Navigator>>>,
    pub route_repo: Arc<Mutex<Box<dyn RoutingRepo>>>
}

impl NavigatorUseCase {
    pub(crate) fn new(navigator: Arc<Mutex<Box<dyn Navigator>>>, route_repo: Arc<Mutex<Box<dyn RoutingRepo>>>) -> Self {
        NavigatorUseCase {
            navigator,
            route_repo
        }
    }

    pub(crate) fn build_graph(&self) {
        let clone = self.navigator.clone();
        thread::spawn(move || {
            clone.lock().expect("nop").build_graph();
        });
    }

    pub(crate) fn calculate_route(&self, route: RouteRequest) -> Option<u32> {
        let clone = self.navigator.clone();
        let repo_clone = self.route_repo.clone();
        let mut job_id = None;
        {
            job_id = Some(self.route_repo.lock().unwrap().get_job_id());
        }
        thread::spawn(move|| {
            let mut result = None;
            { // extra scope to unlock navigator after route is calculated
                let mut nav = clone.lock().unwrap();
                result = nav.calculate_route(route);
            }
            if result.is_some() {
               // save route
                repo_clone.lock().unwrap().add_route(result.unwrap());
            }
        });
        job_id
    }

    pub(crate) fn get_number_nodes(&self) -> u32 {
        self.navigator.lock().unwrap().get_number_nodes()
    }

    pub(crate) fn get_route(&self, id: usize) -> Option<ShipRoute> {
        let mut n = self.route_repo.lock().unwrap();
        n.get_route(id)
    }

    pub(crate) fn get_job_id(&self) -> u32 {
        self.route_repo.lock().unwrap().get_job_id()
    }
}