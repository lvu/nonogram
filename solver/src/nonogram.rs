use common::{invert_value, line_to_str, LineHints, KNOWN, UNKNOWN};
use itertools::Itertools;
use line::{Line, SolveCache};
use ndarray::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io;

mod common;
mod line;

#[derive(Debug)]
pub struct Nonogram {
    field: Array2<u8>,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

type SolutionResult = HashSet<String>;

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
    pub fn solve_by_lines(&mut self, cache: &mut SolveCache) -> bool {
        for row_idx in 0..self.row_hints.len() {
            let mut line = self.row_line(row_idx);
            if line.solve(cache).is_none() {
                return false
            }
        }

        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.col_hints.len());
        let mut changed_rows: HashSet<usize> = HashSet::with_capacity(self.field.nrows());
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                let mut line = self.col_line(col_idx);
                match line.solve(cache) {
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
                match line.solve(cache) {
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

    fn do_solve(&mut self, depth: usize, cache: &mut SolveCache) -> SolutionResult {
        let mut result: SolutionResult = HashSet::new();
        let all_coords: Vec<(usize, usize)> = self.field
            .indexed_iter()
            .filter_map(|(coords, &val)| match val {UNKNOWN => Some(coords), _ => None})
            .collect();
        let all_assumption_values: Vec<Vec<u8>> = (0..depth).map(|_| KNOWN.into_iter()).multi_cartesian_product().collect();
        let mut backup_field = self.field.clone();
        let num_cells = self.field.len();
        let num_cell_entries = (num_cells - depth + 1 .. num_cells).fold(1, |p, x| 2 * p * x);
        let mut cell_possibilities = all_coords
            .iter()
            .cartesian_product(KNOWN.into_iter())
            .map(|(&c, v)| ((c, v), num_cell_entries))
            .collect::<HashMap<((usize, usize), u8), usize>>();

        for assumption_coords in all_coords.iter().combinations(depth) {
            for assumption_values in all_assumption_values.iter() {
                for (&&coords, &val) in assumption_coords.iter().zip(assumption_values.iter()) {
                    self.field[coords] = val;
                }
                if !self.solve_by_lines(cache) {
                    for (&&coords, &val) in assumption_coords.iter().zip(assumption_values.iter()) {
                        let num_possibilities = cell_possibilities.get_mut(&(coords, val)).unwrap();
                        debug_assert!(*num_possibilities > 0);
                        *num_possibilities -= 1;
                        if *num_possibilities == 0 {
                            backup_field[coords] = invert_value(val);
                            println!("Found new at {coords:?}: not {val}");
                        }
                    }
                } else if self.is_solved() {
                    result.insert(self.field_as_string());
                }
                self.field.assign(&backup_field);
            }
        }
        result
    }

    pub fn solve(&mut self, max_depth: usize) -> Result<SolutionResult, String> {
        let mut cache: SolveCache = HashMap::new();
        if !self.solve_by_lines(&mut cache) {
            return Err("Controversial puzzle".to_string());
        }
        if self.is_solved() {
            return Ok(HashSet::from([self.field_as_string()]));
        }
        for depth in 1..max_depth + 1 {
            println!("Depth {} didn't work, trying further...", depth - 1);
            let result = self.do_solve(depth, &mut cache);
            if result.len() > 0 {
                return Ok(result);
            }
        }
        Err("No solution found".to_string())
    }
}

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