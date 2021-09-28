/* tslint:disable */
/* eslint-disable */
import { Node } from './node';
export interface BenchmarkResult {
  amount_nodes_popped: number;
  distance: number;
  end_node: Node;
  nmb_nodes: number;
  query_id: number;
  start_node: Node;
  time: number;
}
