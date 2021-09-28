/* tslint:disable */
/* eslint-disable */
import { BenchmarkResult } from './benchmark-result';
export interface AlgoBenchmark {
  avg_distance_per_ms: number;
  results: Array<BenchmarkResult>;
}
