use common::{line_to_str, LineHints, UNKNOWN, EMPTY, FILLED};
use itertools::Itertools;
use line::{Line, LineMut, SolveCache, cache_hits, cache_misses};
use ndarray::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io;

mod common;
mod line;

#[derive(Debug)]
pub struct Nonogram {
    field: Array2<i8>,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

type SolutionResult = HashMap<String, Vec<(Vec<(usize, usize)>, Vec<i8>)>>;

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
        self.field.rows().into_iter().map(line_to_str).collect::<Vec<String>>().join("\n")
    }

    fn row_line(&mut self, row_idx: usize) -> RowLine {
        RowLine {nono: self, row_idx}
    }

    fn col_line(&mut self, col_idx: usize) -> ColLine {
        ColLine {nono: self, col_idx}
    }

    /// Solves the nonogram in-place only looking at a sinngle line at a time.
    /// Returns false if there was a controversy.
    ///
    /// The complete solution isn't guaranteed, the nonogram may be solved only partially.
    ///
    /// If there was a controversy, the field's contents is undefined.
    pub fn solve_by_lines(&mut self, cache: &mut SolveCache) -> bool {
        for row_idx in 0..self.row_hints.len() {
            if self.row_line(row_idx).solve(cache).is_none() {
                return false
            }
        }

        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.col_hints.len());
        let mut changed_rows: HashSet<usize> = HashSet::new();
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                match self.col_line(col_idx).solve(cache) {
                    Some(ch) => changed_rows.extend(ch.iter()),
                    None => return false
                }
            }
            if changed_rows.is_empty() {
                return true
            }

            changed_cols.clear();
            for &row_idx in changed_rows.iter() {
                match self.row_line(row_idx).solve(cache) {
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
        let mut result: SolutionResult = HashMap::new();
        let all_coords: Vec<(usize, usize)> = self.field
            .indexed_iter()
            .filter_map(|(coords, &val)| match val {UNKNOWN => Some(coords), _ => None})
            .collect();
        let all_assumption_values: Vec<Vec<i8>> = (0..depth).map(|_| [EMPTY, FILLED].into_iter()).multi_cartesian_product().collect();
        let backup_field = self.field.clone();

        for assumption_coords in all_coords.iter().combinations(depth) {
            for assumption_values in all_assumption_values.iter() {
                for (&&coords, &val) in assumption_coords.iter().zip(assumption_values.iter()) {
                    self.field[coords] = val;
                }
                if self.solve_by_lines(cache) && self.is_solved() {
                    let result_val = result
                        .entry(self.field_as_string())
                        .or_insert_with(|| Vec::new());
                    result_val.push((assumption_coords.iter().map(|&&x| x).collect(), assumption_values.clone()));
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
            return Ok(HashMap::from([(self.field_as_string(), Vec::new())]));
        }
        for depth in 1..max_depth + 1 {
            println!("Depth {} didn't work, trying further...", depth - 1);
            let result = self.do_solve(depth, &mut cache);
            if result.len() > 0 {
                return Ok(result);
            }:while {

            }
        }
        unsafe {println!("Hits: {cache_hits}, misses: {cache_misses}");}
        Err("No solution found".to_string())
    }
}

const REV_VALUES: [(i8, i8); 2] = [(EMPTY, FILLED), (FILLED, EMPTY)];

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

struct RowLine<'a> {
    nono: &'a mut Nonogram,
    row_idx: usize
}

impl<'a> Line for RowLine<'a> {
    fn hints(&self) -> &LineHints {
        &self.nono.row_hints[self.row_idx]
    }

    fn cells(&self) -> ArrayView1<i8> {
        self.nono.field.row(self.row_idx)
    }
}

impl<'a> LineMut for RowLine<'a> {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8> {
        self.nono.field.row_mut(self.row_idx)
    }
}

struct ColLine<'a> {
    nono: &'a mut Nonogram,
    col_idx: usize
}

impl<'a> Line for ColLine<'a> {
    fn hints(&self) -> &LineHints {
        &self.nono.col_hints[self.col_idx]
    }

    fn cells(&self) -> ArrayView1<i8> {
        self.nono.field.column(self.col_idx)
    }
}

impl<'a> LineMut for ColLine<'a> {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8> {
        self.nono.field.column_mut(self.col_idx)
    }
}

mod tests {
    use super::*;

    #[test]
    fn solve_by_line_simple() {
        let mut nono = Nonogram::from_hints(
            vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]]
        );
        nono.solve_by_lines();
        let row_strs: Vec<String> = (0..nono.row_hints.len()).map(|idx| nono.row_line(idx).to_string()).collect();
        assert_eq!(row_strs, vec![
            "*****",
            "*XXXX",
            "*****",
            "XXXX*",
            "*****"
        ]);
    }


}