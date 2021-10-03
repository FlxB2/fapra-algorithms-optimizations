use std::fmt;
use std::cmp::Ordering;

// heap item used for min heaps
#[derive(Debug)]
pub struct PriorityHeapItem {
    pub(crate) node_id: u32,
    pub(crate) distance: u32,
    pub(crate) priority: u64,
    pub(crate) previous_node: u32,
}

impl fmt::Display for PriorityHeapItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        write!(f, "node_id: {}, distance: {}, previous_node: {}", self.node_id, self.distance, self.previous_node)
    }
}

impl PartialEq for PriorityHeapItem {
    fn eq(&self, other: &Self) -> bool {
        other.priority.eq(&self.priority)
    }
}

impl Eq for PriorityHeapItem {}

impl PartialOrd for PriorityHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.priority.cmp(&self.priority))
    }
}

impl Ord for PriorityHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}
