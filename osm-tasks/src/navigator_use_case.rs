use crate::persistence::navigator::Navigator;
use std::sync::{Mutex, Arc};
use crate::persistence::in_memory_routing_repo::{RouteRequest, ShipRoute};
use std::thread;
use crate::persistence::routing_repo::RoutingRepo;
use crate::persistence::benchmark_repo::BenchmarkRepo;
use crate::model::benchmark::CollectedBenchmarks;

pub struct NavigatorUseCase {
    pub navigator: Arc<Mutex<Box<dyn Navigator>>>,
    pub route_repo: Arc<Mutex<Box<dyn RoutingRepo>>>,
    pub benchmark_repo: Arc<Mutex<Box<dyn BenchmarkRepo>>>
}

impl NavigatorUseCase {
    pub(crate) fn new(navigator: Arc<Mutex<Box<dyn Navigator>>>, route_repo: Arc<Mutex<Box<dyn RoutingRepo>>>, benchmark_repo: Arc<Mutex<Box<dyn BenchmarkRepo>>>) -> Self {
        NavigatorUseCase {
            navigator,
            route_repo,
            benchmark_repo
        }
    }

    pub(crate) fn build_graph(&self) {
        let clone = self.navigator.clone();
        thread::spawn(move || {
            clone.lock().expect("could not lock graph").build_graph();
        });
    }

    pub(crate) fn calculate_route(&self, route: RouteRequest) -> Option<u32> {
        if self.get_number_nodes() == 0 {
            return None;
        }
        let clone = self.navigator.clone();
        let repo_clone = self.route_repo.clone();
        let job_id;
        {
            job_id = Some(self.route_repo.lock().unwrap().get_job_id());
        }
        thread::spawn(move || {
            let result;
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

    pub(crate) fn benchmark(&self, nmb_queries: usize) {
        let benchmark_repo = self.benchmark_repo.clone();
        let navigator_clone = self.navigator.clone();
        thread::spawn(move || {
            let result;
            {
                result = navigator_clone.lock().expect("could not lock graph").run_benchmarks(nmb_queries)
            }
            benchmark_repo.lock().unwrap().set_results(result);
        });
    }

    pub(crate) fn get_benchmark_results(&self) -> Option<CollectedBenchmarks> {
        let lock = self.benchmark_repo.lock();
        if lock.is_ok() {
            return Some(lock.unwrap().get_results());
        }
        None
    }

    pub(crate) fn is_benchmark_finished(&self) -> bool {
        let lock = self.benchmark_repo.lock();
        if lock.is_err() {
            return false;
        }
        lock.unwrap().is_finished()
    }

    pub(crate) fn get_number_nodes(&self) -> u32 {
        self.navigator.lock().unwrap().get_number_nodes()
    }

    pub(crate) fn get_route(&self, id: usize) -> Option<ShipRoute> {
        if self.get_number_nodes() == 0 {
            return None;
        }
        let n = self.route_repo.lock().unwrap();
        n.get_route(id)
    }

    pub(crate) fn test_ch(&self) {
        self.navigator.lock().unwrap().test_ch();
    }

    #[allow(dead_code)]
    pub(crate) fn get_job_id(&self) -> u32 {
        self.route_repo.lock().unwrap().get_job_id()
    }
}
