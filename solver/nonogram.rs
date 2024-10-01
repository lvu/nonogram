use ahash::AHasher;
use assumption::Assumption;
use common::{CellValue, LineHints, Unknown, KNOWN};
use field::Field;
use itertools::Itertools;
use line::{Line, LineCache, LineType};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::io;
use LineType::*;

mod assumption;
mod common;
mod field;
mod line;

type MultiSolution = HashMap<Vec<CellValue>, Field>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SolutionResult {
    Controversial,
    Unsolved(Vec<Assumption>),
    Solved(MultiSolution),
}

pub use SolutionResult::*;

type ABuildHasher = BuildHasherDefault<AHasher>;

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
}

pub struct Solver {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
    row_cache: Vec<LineCache<ABuildHasher>>,
    col_cache: Vec<LineCache<ABuildHasher>>,
    max_depth: usize,
    find_all: bool,
}

impl Solver {
    pub fn from_reader<R: io::Read>(
        rdr: R,
        max_depth: usize,
        find_all: bool,
    ) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(
            descr.row_hints,
            descr.col_hints,
            max_depth,
            find_all,
        ))
    }

    fn from_hints(
        row_hints: Vec<LineHints>,
        col_hints: Vec<LineHints>,
        max_depth: usize,
        find_all: bool,
    ) -> Self {
        let row_cache = (0..row_hints.len())
            .map(|_| RefCell::new(HashMap::default()))
            .collect();
        let col_cache = (0..col_hints.len())
            .map(|_| RefCell::new(HashMap::default()))
            .collect();
        Self { row_hints, col_hints, row_cache, col_cache, max_depth, find_all }
    }

    pub fn create_field(&self) -> Field {
        Field::new(self.nrows(), self.ncols())
    }

    fn nrows(&self) -> usize {
        self.row_hints.len()
    }

    fn ncols(&self) -> usize {
        self.col_hints.len()
    }

    fn row_line<'a>(&'a self, field: &'a Field, row_idx: usize) -> Line {
        Line::new(Row, row_idx, &self.row_hints[row_idx], field.row(row_idx))
    }

    fn col_line<'a>(&'a self, field: &'a Field, col_idx: usize) -> Line {
        Line::new(Col, col_idx, &self.col_hints[col_idx], field.col(col_idx))
    }

    fn line<'a>(&'a self, field: &'a Field, line_type: LineType, line_idx: usize) -> Line {
        match line_type {
            Row => self.row_line(field, line_idx),
            Col => self.col_line(field, line_idx),
        }
    }

    fn cache(&self, line_type: LineType, line_idx: usize) -> &LineCache<ABuildHasher> {
        match line_type {
            Row => &self.row_cache[line_idx],
            Col => &self.col_cache[line_idx],
        }
    }

    fn do_solve_by_lines_step(
        &self,
        field: &mut Cow<Field>,
        line_type: LineType,
        line_changes: &[u8],
    ) -> Option<Vec<Assumption>> {
        let mut all_changes: Vec<Assumption> = Vec::new();
        for line_idx in line_changes.iter().enumerate().filter_map(|(idx, &val)| if val > 0 { Some(idx) } else { None }) {
            let mut line = self.line(&field, line_type, line_idx);
            match line.solve(self.cache(line_type, line_idx)).as_ref() {
                Some(changes) if !changes.is_empty() => {
                    apply_changes(changes, field.to_mut(), &mut all_changes);
                }
                None => return None,
                _ => (),
            }
        }
        Some(all_changes)
    }

    fn do_solve_by_lines(&self, field: &Field, changed_rows: &[u8], changed_cols: &[u8]) -> SolutionResult {
        let mut field = Cow::Borrowed(field);
        let mut all_changes: Vec<Assumption> = Vec::new();
        match self.do_solve_by_lines_step(&mut field, Row, changed_rows) {
            None => return Controversial,
            Some(changes) => all_changes.extend(changes),
        }
        let mut line_type = Col;
        let mut changed_idxs = Vec::from(changed_cols);
        loop {
            match self.do_solve_by_lines_step(&mut field, line_type, &changed_idxs) {
                None => return Controversial,
                Some(changes) => {
                    if changes.is_empty() {
                        return if field.is_solved() {
                            Solved(HashMap::from([(field.key(), field.into_owned())]))
                        } else {
                            Unsolved(all_changes)
                        };
                    }
                    changed_idxs.clear();
                    changed_idxs.resize(match line_type { Row => self.nrows(), Col => self.ncols() }, 0);
                    for ass in changes.iter() {
                        changed_idxs[ass.line_idx(line_type.other())] += 1;
                    }
                    all_changes.extend(changes);
                }
            }
            line_type = line_type.other();
        }
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.nrows()).cartesian_product(0..self.ncols())
    }

    fn do_step(&self, field: &Field, max_depth: usize) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        let mut solutions = HashMap::new();
        let mut has_unsolved = false;
        let mut changed_rows = vec![0u8; self.nrows()];
        let mut changed_cols = vec![0u8; self.ncols()];
        for coords in self.iter_coords() {
            if field.get(coords) != Unknown {
                continue;
            }
            let mut has_controversy = false;
            for val in KNOWN {
                let ass = Assumption { coords, val };
                ass.apply(&mut field);
                changed_rows[ass.coords.0] += 1;
                changed_cols[ass.coords.1] += 1;
                match self.do_solve(&field, max_depth, &changed_rows, &changed_cols) {
                    Solved(res) => {
                        extend_solutions_from(&mut solutions, res);
                        if !self.find_all {
                            return Solved(solutions);
                        }
                        ass.unapply(&mut field);
                        changed_rows[ass.coords.0] -= 1;
                        changed_cols[ass.coords.1] -= 1;
                    }
                    Unsolved(_) => {
                        has_unsolved = true;
                        ass.unapply(&mut field);
                        changed_rows[ass.coords.0] -= 1;
                        changed_cols[ass.coords.1] -= 1;
                    }
                    Controversial => {
                        if has_controversy {
                            return Controversial;
                        }
                        ass.invert().apply(&mut field);
                        all_changes.push(ass.invert());
                        has_controversy = true;
                    }
                }
            }
        }
        if !solutions.is_empty() && !(has_unsolved && self.find_all) {
            Solved(solutions)
        } else {
            Unsolved(all_changes)
        }
    }

    fn do_solve(&self, field: &Field, max_depth: usize, changed_rows: &[u8], changed_cols: &[u8]) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        let mut changed_rows = Cow::Borrowed(changed_rows);
        let mut changed_cols = Cow::Borrowed(changed_cols);

        'outer: loop {
            let by_lines = self.do_solve_by_lines(&field, changed_rows.as_ref(), changed_cols.as_ref());
            match by_lines {
                Controversial | Solved(_) => return by_lines,
                Unsolved(changes) => {
                    if max_depth == 0 {
                        return Unsolved(changes);
                    }
                    apply_changes(&changes, &mut field, &mut all_changes);
                }
            }

            changed_rows.to_mut().iter_mut().for_each(|v| *v=0);
            changed_cols.to_mut().iter_mut().for_each(|v| *v=0);
            for depth in 0..max_depth {
                let by_step = self.do_step(&field, depth);
                match by_step {
                    Solved(_) | Controversial => return by_step,
                    Unsolved(changes) => {
                        if !changes.is_empty() {
                            apply_changes(&changes, &mut field, &mut all_changes);
                            for ass in changes {
                                changed_rows.to_mut()[ass.coords.0] += 1;
                                changed_cols.to_mut()[ass.coords.1] += 1;
                            }
                            continue 'outer;
                        }
                    }
                }
            }
            return Unsolved(all_changes)
        }
    }

    pub fn solve(&self) -> SolutionResult {
        self.do_solve(
            &self.create_field(),
            self.max_depth,
            &vec![1; self.nrows()],
            &vec![1; self.ncols()],
        )
    }
}

fn apply_changes(changes: &[Assumption], field: &mut Field, all_changes: &mut Vec<Assumption>) {
    all_changes.extend_from_slice(&changes);
    changes.iter().for_each(|ass| ass.apply(field));
}

fn extend_solutions_from(soluions: &mut MultiSolution, new_solutions: MultiSolution) {
    new_solutions.into_iter().for_each(|(k, v)| {
        soluions.entry(k).or_insert(v);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    impl SolutionResult {
        fn assert_solved(&self, results: &[&str]) {
            if let Solved(flds) = self {
                assert_eq!(
                    flds.iter().map(|(_, f)| f.to_string()).collect::<HashSet<String>>(),
                    results.iter().map(|x| x.to_string()).collect::<HashSet<String>>()
                );
            } else {
                panic!("Not solved: {self:?}");
            }
        }
    }

    #[test]
    fn solve_by_line() {
        let solver = Solver::from_hints(
            vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]],
            0,
            false,
        );
        solver.solve().assert_solved(&["\
                #####\n\
                #....\n\
                #####\n\
                ....#\n\
                #####\n\
        "]);
    }

    #[test]
    fn solve_ambiguous() {
        let solver = Solver::from_hints(vec![vec![1], vec![1]], vec![vec![1], vec![1]], 3, true);
        solver.solve().assert_solved(&[
            "#.\n\
             .#\n",
            ".#\n\
             #.\n",
        ]);
    }


    #[test]
    fn solve_double_ambiguous_naive() {
        let solver = Solver::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            2,
            true,
        );
        solver.solve().assert_solved(&[
            "#..#.\n\
             .#..#\n",
            "#...#\n\
             .#.#.\n",
            ".#..#\n\
             #..#.\n",
            ".#.#.\n\
             #...#\n",
        ]);
    }
}
