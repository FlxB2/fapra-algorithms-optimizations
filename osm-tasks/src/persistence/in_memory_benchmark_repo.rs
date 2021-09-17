use crate::benchmark::{CollectedBenchmarks};
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
                results: HashMap::new()
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
