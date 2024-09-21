use super::common::{CellValue, Unknown};
use super::reachability_graph::ReachabilityGraph;
use super::Field;
use itertools::Itertools;
use std::iter;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub struct Assumption {
    pub coords: (usize, usize),
    pub val: CellValue,
}

impl Assumption {
    pub fn invert(&self) -> Self {
        Self { coords: self.coords, val: self.val.invert() }
    }

    pub fn apply(&self, field: &mut Field) {
        field.set(self.coords, self.val);
    }

    pub fn unapply(&self, field: &mut Field) {
        field.set(self.coords, Unknown);
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

    pub fn get_impossible(&self) -> impl Iterator<Item = &Assumption> {
        self.strongly_connected_components()
            .into_iter()
            .filter(|scc| self.is_impossible(scc[0]))
            .flatten()
    }
}
