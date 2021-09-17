use crate::benchmark::CollectedBenchmarks;

pub trait BenchmarkRepo: Send + Sync {
    fn new() -> Self
        where
            Self: Sized;
    fn is_finished(&self) -> bool;
    fn get_results(&self) -> CollectedBenchmarks;
    fn set_results(&mut self, results: CollectedBenchmarks);
}
