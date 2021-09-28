use crate::model::benchmark::{CollectedBenchmarks, AlgoBenchmark};
use crate::persistence::benchmark_repo::BenchmarkRepo;
use std::collections::HashMap;

pub(crate) struct InMemoryBenchmarkRepo {
    benchmarks: CollectedBenchmarks,
    finished: bool
}

impl BenchmarkRepo for InMemoryBenchmarkRepo {
    fn new() -> InMemoryBenchmarkRepo {
        InMemoryBenchmarkRepo {
            benchmarks: CollectedBenchmarks {
                dijkstra: AlgoBenchmark::new(),
                a_star: AlgoBenchmark::new(),
                bd_dijkstra: AlgoBenchmark::new()
            },
            finished: false
        }
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_results(&self) -> CollectedBenchmarks {
        self.benchmarks.clone()
    }

    fn set_results(&mut self, results: CollectedBenchmarks) {
        self.benchmarks = results;
        self.finished = true;
    }
}
