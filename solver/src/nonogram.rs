use common::{invert_value, line_to_str, LineHints, KNOWN, UNKNOWN};
use itertools::Itertools;
use line::{Line, LineCache};
use ndarray::prelude::*;
use reachability_graph::ReachabilityGraph;
use std::collections::{HashMap, HashSet};
use std::{io};

mod common;
mod line;
mod reachability_graph;

type MultiSolution = HashSet<String>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MultiSolutionResult {
    Controversial,
    Unsolved,
    Solved(MultiSolution)
}

pub use MultiSolutionResult::*;

pub struct Nonogram {
    field: Field,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

impl Nonogram {
    pub fn from_reader<R: io::Read>(rdr: R) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(descr.row_hints, descr.col_hints))
    }

    fn from_hints(row_hints: Vec<LineHints>, col_hints: Vec<LineHints>) -> Self {
        Self {
            field: Array::from_elem((row_hints.len(), col_hints.len()), UNKNOWN),
            row_hints, col_hints
        }
    }

    pub fn field_as_string(&self) -> String {
        self.field.rows().into_iter().map(|x| line_to_str(&x)).collect::<Vec<String>>().join("\n")
    }

    fn row_line(&mut self, row_idx: usize) -> Line {
        Line {
            hints: &self.row_hints[row_idx],
            cells: self.field.row_mut(row_idx)
        }
    }

    fn col_line(&mut self, col_idx: usize) -> Line {
        Line {
            hints: &self.col_hints[col_idx],
            cells: self.field.column_mut(col_idx)
        }
    }

    /// Solves the nonogram in-place only looking at a sinngle line at a time.
    /// Returns false if there was a controversy.
    ///
    /// The complete solution isn't guaranteed, the nonogram may be solved only partially.
    ///
    /// If there was a controversy, the field's contents is undefined.
    pub fn solve_by_lines(&mut self, line_cache: &mut LineCache) -> bool {
        for row_idx in 0..self.row_hints.len() {
            let mut line = self.row_line(row_idx);
            if line.solve(line_cache).is_none() {
                return false
            }
        }

        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.col_hints.len());
        let mut changed_rows: HashSet<usize> = HashSet::with_capacity(self.field.nrows());
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                let mut line = self.col_line(col_idx);
                match line.solve(line_cache) {
                    Some(ch) => changed_rows.extend(ch.iter()),
                    None => return false
                }
            }
            if changed_rows.is_empty() {
                return true
            }

            changed_cols.clear();
            for &row_idx in changed_rows.iter() {
                let mut line = self.row_line(row_idx);
                match line.solve(line_cache) {
                    Some(ch) => changed_cols.extend(ch.iter()),
                    None => return false
                }
            }
            if changed_cols.is_empty() {
                return true
            }
        }
    }

    fn is_solved(&self) -> bool {
        self.field.iter().all(|&x| x != UNKNOWN)
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.field.nrows()).cartesian_product(0..self.field.ncols())
    }

    fn iter_assumptions(&self) -> impl Iterator<Item = Assumption> {
        self.iter_coords().flat_map(|coords| KNOWN.iter().map(move |&val| Assumption {coords, val}))
    }

    fn do_solve(
        &mut self,
        find_all: bool,
        depth: Option<usize>,
        assumptions: &Vec<Assumption>,
        line_cache: &mut LineCache
    ) -> MultiSolutionResult {
        if let Some(d) = depth { if d == 0 {
            return Unsolved
        }}
        if !self.solve_by_lines(line_cache) {
            // println!("Controversy at {assumptions:?}");
            return Controversial;
        }
        if self.is_solved() {
            return Solved(HashSet::from([self.field_as_string()]));
        }

        let mut solutions = HashSet::new();
        let mut backup_field = self.field.clone();
        let mut new_assumptions = assumptions.clone();
        new_assumptions.push(Assumption::default());
        let mut prev_controversial: Option<Assumption> = None;
        let mut has_unsolved = false;
        for ass in self.iter_assumptions() {
            if self.field[ass.coords] != UNKNOWN {
                continue;
            }
            ass.apply(&mut self.field);
            let mut changed = false;
            new_assumptions[assumptions.len()] = ass.clone();
            match self.do_solve(find_all, depth.map(|d| d - 1), &new_assumptions, line_cache) {
                Solved(res) => {
                    solutions.extend(res);
                    if !find_all {
                        return Solved(solutions);
                    }
                },
                Unsolved => { has_unsolved = true; },
                Controversial => {
                    if let Some(prev_ass) = prev_controversial {
                        if prev_ass.coords == ass.coords {
                            return Controversial;
                        }
                    }
                    ass.invert().apply(&mut backup_field);
                    prev_controversial = Some(ass);
                    changed = true;
                }
            }
            self.field.assign(&backup_field);
            if changed {
                self.solve_by_lines(line_cache);
                backup_field.assign(&self.field);
            }
        }
        if solutions.is_empty() || (has_unsolved && find_all) { Unsolved } else { Solved(solutions) }
    }

    pub fn solve(&mut self, max_depth: Option<usize>, find_all: bool) -> MultiSolutionResult {
        let mut line_cache: LineCache = HashMap::new();
        self.do_solve(find_all, max_depth, &Vec::new(), &mut line_cache)
    }

    pub fn solve_2sat(&mut self) -> MultiSolutionResult {
        let mut line_cache: LineCache = HashMap::new();
        if !self.solve_by_lines(&mut line_cache) {
            return Controversial;
        }
        println!("{}\n", self.field_as_string());
        if self.is_solved() {
            return Solved(HashSet::from([self.field_as_string()]));
        }

        loop {
            let mut reach: ReachabilityGraph<Assumption> = ReachabilityGraph::new();
            let old_field = self.field.clone();
            let backup_field = self.field.clone();
            let mut solutions = HashSet::new();
            let mut has_unsolved = false;
            for ass1 in self.iter_assumptions() {
                if self.field[ass1.coords] != UNKNOWN {
                    continue;
                }
                for ass2 in self.iter_assumptions() {
                    if ass1.coords <= ass2.coords
                    || self.field[ass2.coords] != UNKNOWN
                    || reach.is_reachable(&ass1, &ass2.invert()) {
                        continue;
                    }
                    ass1.apply(&mut self.field);
                    ass2.apply(&mut self.field);
                    if !self.solve_by_lines(&mut line_cache) {
                        reach.set_reachable(&ass1, &ass2.invert());
                        reach.set_reachable(&ass2, &ass1.invert());
                    } else if self.is_solved() {
                        solutions.insert(self.field_as_string());
                    } else {
                        has_unsolved = true;
                    }
                    self.field.assign(&backup_field);
                }
            }
            for ass in reach.get_impossible() {
                ass.invert().apply(&mut self.field);
            }
            self.solve_by_lines(&mut line_cache);
            if old_field == self.field {
                return if has_unsolved { Unsolved } else { Solved(solutions) }
            }
            println!("{}\n", self.field_as_string());
        }
    }
}

type Field = Array2<u8>;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
struct Assumption {
    coords: (usize, usize),
    val: u8
}

impl Assumption {
    fn invert(&self) -> Self {
        Self {
            coords: self.coords,
            val: invert_value(self.val)
        }
    }

    fn apply(&self, field: &mut Field) {
        field[self.coords] = self.val;
    }
}

impl ReachabilityGraph<Assumption> {
    fn is_impossible(&self, node: &Assumption) -> bool {
        let mut reachable: Vec<&Assumption> = self.get_reachable(node).unwrap().collect();
        reachable.sort_unstable_by_key(|a| a.coords);
        for (a, b) in reachable.iter().tuple_windows() {
            if a.coords == b.coords {
                return true;
            }
        }
        false
    }

    fn get_impossible(&self) -> Vec<&Assumption> {
        let mut result: Vec<&Assumption> = Vec::new();
        for scc in self.strongly_connected_components().iter() {
            if self.is_impossible(scc[0]) {
                result.extend(scc.iter());
            }
        }
        result
    }
}

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_by_line_simple() {
        let mut nono = Nonogram::from_hints(
            vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]]
        );
        nono.solve_by_lines(&mut HashMap::new());
        assert_eq!(nono.field_as_string(), vec![
            "*****",
            "*XXXX",
            "*****",
            "XXXX*",
            "*****"
        ].join("\n"));
    }

    #[test]
    fn solve_ambiguous_recursive() {
        let mut nono = Nonogram::from_hints(
            vec![vec![1], vec![1]],
            vec![vec![1], vec![1]]
        );
        let result = nono.solve(Some(3), true);
        assert_eq!(result, Solved(HashSet::from([
            "*X\n\
             X*".to_string(),
            "X*\n\
             *X".to_string()
        ])));
    }

    #[test]
    fn solve_ambiguous_2sat() {
        let mut nono = Nonogram::from_hints(
            vec![vec![1], vec![1]],
            vec![vec![1], vec![1]]
        );
        let result = nono.solve_2sat();
        assert_eq!(result, Solved(HashSet::from([
            "*X\n\
             X*".to_string(),
            "X*\n\
             *X".to_string()
        ])));
    }

    #[test]
    fn solve_double_ambiguous_recursive() {
        let mut nono = Nonogram::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]]
        );
        let result = nono.solve(Some(3), true);
        assert_eq!(result, Solved(HashSet::from([
            "*XX*X\n\
             X*XX*".to_string(),
            "*XXX*\n\
             X*X*X".to_string(),
            "X*XX*\n\
             *XX*X".to_string(),
            "X*X*X\n\
             *XXX*".to_string()
        ])));
    }

    #[test]
    fn solve_double_ambiguous_2sat() {
        let mut nono = Nonogram::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]]
        );
        let result = nono.solve_2sat();
        assert_eq!(result, Solved(HashSet::from([
            "*XX*X\n\
             X*XX*".to_string(),
            "*XXX*\n\
             X*X*X".to_string(),
            "X*XX*\n\
             *XX*X".to_string(),
            "X*X*X\n\
             *XXX*".to_string()
        ])));
    }
}