use std::fmt;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct HeapItem {
    pub(crate) node_id: u32,
    pub(crate) distance: u32,
    pub(crate) previous_node: u32,
}

impl fmt::Display for HeapItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        write!(f, "node_id: {}, distance: {}, previous_node: {}", self.node_id, self.distance, self.previous_node)
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        other.distance.eq(&self.distance)
    }
}

impl Eq for HeapItem {}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.distance.cmp(&self.distance))
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}
