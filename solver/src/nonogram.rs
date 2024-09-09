use common::{invert_value, line_to_str, LineHints, KNOWN, UNKNOWN};
use itertools::Itertools;
use line::{Line, LineCache};
use ndarray::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io;

mod common;
mod line;

type MultiSolution = HashSet<String>;

#[derive(Clone)]
pub enum MultiSolutionResult {
    Controversial,
    Unsolved,
    Solved(MultiSolution)
}

pub use MultiSolutionResult::*;

#[derive(Debug)]
pub struct Nonogram {
    field: Array2<u8>,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
    cache_hits: usize,
    cache_misses: usize
}

impl Nonogram {
    pub fn from_reader<R: io::Read>(rdr: R) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(descr.row_hints, descr.col_hints))
    }

    fn from_hints(row_hints: Vec<LineHints>, col_hints: Vec<LineHints>) -> Self {
        Self {
            field: Array::from_elem((row_hints.len(), col_hints.len()), UNKNOWN),
            row_hints, col_hints, cache_hits: 0, cache_misses: 0
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

    fn do_solve(
        &mut self,
        find_all: bool,
        depth: Option<usize>,
        line_cache: &mut LineCache
    ) -> MultiSolutionResult {
        if let Some(d) = depth { if d == 0 {
            return Unsolved
        }}
        if !self.solve_by_lines(line_cache) {
            return Controversial;
        }
        if self.is_solved() {
            return Solved(HashSet::from([self.field_as_string()]));
        }

        let mut result = HashSet::new();
        let mut backup_field = self.field.clone();
        for coords in self.iter_coords() {
            if self.field[coords] != UNKNOWN {
                continue;
            }
            let mut num_controversial: u8 = 0;
            for val in KNOWN.into_iter() {
                self.field[coords] = val;
                match self.do_solve(find_all, depth.map(|d| d - 1), line_cache) {
                    Solved(res) => {
                        result.extend(res);
                        if !find_all {
                            return Solved(result);
                        }
                    },
                    Unsolved => (),
                    Controversial => {
                        num_controversial += 1;
                        backup_field[coords] = invert_value(val);
                    }
                }
                self.field.assign(&backup_field);
            }
            if num_controversial == 2 {
                return Controversial;
            }
        }
        if result.is_empty() { Unsolved } else { Solved(result) }
    }

    pub fn solve(&mut self, max_depth: Option<usize>, find_all: bool) -> MultiSolutionResult {
        let mut line_cache: LineCache = HashMap::new();
        self.do_solve(find_all, max_depth, &mut line_cache)
    }
}

#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
struct Assumption {
    coords: (usize, usize),
    val: u8
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
}