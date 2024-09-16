use itertools::Itertools;
use super::Field;
use super::common::{invert_value, UNKNOWN};
use super::reachability_graph::ReachabilityGraph;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub struct Assumption {
    pub coords: (usize, usize),
    pub val: u8
}

impl Assumption {
    pub fn invert(&self) -> Self {
        Self {
            coords: self.coords,
            val: invert_value(self.val)
        }
    }

    pub fn apply(&self, field: &mut Field) {
        field.set(self.coords, self.val);
    }

    pub fn unapply(&self, field: &mut Field) {
        field.set(self.coords, UNKNOWN);
    }
}

impl ReachabilityGraph<Assumption> {
    pub fn is_impossible(&self, node: &Assumption) -> bool {
        let mut reachable: Vec<&Assumption> = self.get_reachable(node).unwrap().collect();
        reachable.sort_unstable_by_key(|a| a.coords);
        for (a, b) in reachable.iter().tuple_windows() {
            if a.coords == b.coords {
                return true;
            }
        }
        false
    }

    pub fn get_impossible(&self) -> Vec<&Assumption> {
        let mut result: Vec<&Assumption> = Vec::new();
        for scc in self.strongly_connected_components().iter() {
            if self.is_impossible(scc[0]) {
                result.extend(scc.iter());
            }
        }
        result
    }
}
