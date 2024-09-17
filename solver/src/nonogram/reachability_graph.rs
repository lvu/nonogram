use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct ReachabilityGraph<T> {
    nodes: Vec<T>,
    rev_nodes: HashMap<T, usize>,
    // reachability_in[A].contains(B) means that A is reachable from B
    reachability_in: Vec<HashSet<usize>>,
    // reachability_out[B].contains(A) means that B is reachable from A
    reachability_out: Vec<HashSet<usize>>,
}

impl<T: Hash + Eq + Clone> ReachabilityGraph<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            rev_nodes: HashMap::new(),
            reachability_in: Vec::new(),
            reachability_out: Vec::new(),
        }
    }

    fn add_node(&mut self, node: T) -> usize {
        debug_assert!(!self.rev_nodes.contains_key(&node));
        let node_idx = self.nodes.len();
        self.nodes.push(node.clone());
        self.rev_nodes.insert(node, node_idx);
        self.reachability_in.push(HashSet::from([node_idx]));
        self.reachability_out.push(HashSet::from([node_idx]));
        node_idx
    }

    fn get_node_idx_or_add(&mut self, node: &T) -> usize {
        self.rev_nodes
            .get(node)
            .map(|&x| x)
            .unwrap_or_else(|| self.add_node(node.clone()))
    }

    /// Add information that b is reachable from a
    pub fn set_reachable(&mut self, a: &T, b: &T) {
        debug_assert!(a != b);
        let a = self.get_node_idx_or_add(a);
        let b = self.get_node_idx_or_add(b);
        let nodes_from: Vec<usize> = self.reachability_in[a].iter().map(|&x| x).collect();
        let nodes_to: Vec<usize> = self.reachability_out[b].iter().map(|&x| x).collect();
        for src in nodes_from {
            for &dst in nodes_to.iter() {
                self.reachability_in[dst].insert(src);
                self.reachability_out[src].insert(dst);
            }
        }
        // println!("{a} -> {b}: {:?} {:?} {:?}", self.nodes, self.);
    }

    pub fn is_reachable(&self, a: &T, b: &T) -> bool {
        let a = match self.rev_nodes.get(a) {
            Some(x) => x,
            None => return false,
        };
        let b = match self.rev_nodes.get(b) {
            Some(x) => x,
            None => return false,
        };
        self.reachability_out[*a].contains(b)
    }

    pub fn get_reachable(&self, node: &T) -> Option<impl Iterator<Item = &T>> {
        self.rev_nodes.get(node).map(|&node_idx| {
            self.reachability_out
                .get(node_idx)
                .unwrap()
                .iter()
                .map(|&out_idx| &self.nodes[out_idx])
        })
    }

    pub fn strongly_connected_components(&self) -> Vec<Vec<&T>> {
        let mut visited: HashSet<&T> = HashSet::new();
        let mut result: Vec<Vec<&T>> = Vec::new();
        for a in self.nodes.iter() {
            if visited.contains(a) {
                continue;
            }
            let scc: Vec<&T> = self
                .get_reachable(a)
                .unwrap()
                .filter(|&b| self.is_reachable(b, a))
                .collect();
            visited.extend(scc.iter());
            result.push(scc);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // A -> B ------> C -> E -> F
    //      ^        /     ^   /
    //       \      /       \-/
    //        - D <-
    fn reachability() {
        let mut g = ReachabilityGraph::new();
        let a = "A".to_string();
        let b = "B".to_string();
        let c = "C".to_string();
        let d = "D".to_string();
        let e = "E".to_string();
        let f = "F".to_string();
        g.set_reachable(&a, &b);
        g.set_reachable(&b, &c);
        g.set_reachable(&c, &d);
        g.set_reachable(&d, &b);
        g.set_reachable(&c, &e);
        g.set_reachable(&e, &f);
        g.set_reachable(&f, &e);
        let a_reachable: HashSet<&String> = g.get_reachable(&a).unwrap().collect();
        assert_eq!(a_reachable, HashSet::from([&a, &b, &c, &d, &e, &f]));
        let b_reachable: HashSet<&String> = g.get_reachable(&b).unwrap().collect();
        assert_eq!(b_reachable, HashSet::from([&b, &c, &d, &e, &f]));
        let d_reachable: HashSet<&String> = g.get_reachable(&d).unwrap().collect();
        assert_eq!(d_reachable, HashSet::from([&b, &c, &d, &e, &f]));
        let e_reachable: HashSet<&String> = g.get_reachable(&e).unwrap().collect();
        assert_eq!(e_reachable, HashSet::from([&e, &f]));

        if let Some(_) = g.get_reachable(&"G".to_string()) {
            panic!("G exists");
        };

        assert!(g.is_reachable(&a, &e));
        assert!(!g.is_reachable(&e, &a));

        let mut sccs = g.strongly_connected_components();
        for scc in sccs.iter_mut() {
            scc.sort();
        }
        sccs.sort();
        assert_eq!(sccs, vec![vec![&a], vec![&b, &c, &d], vec![&e, &f]])
    }
}
