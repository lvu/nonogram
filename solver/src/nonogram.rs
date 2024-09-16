use assumption::Assumption;
use common::{line_to_str, LineHints, KNOWN, UNKNOWN};
use itertools::Itertools;
use field::Field;
use line::{Line, LineCache};
use reachability_graph::ReachabilityGraph;
use std::collections::{HashMap, HashSet};
use std::{io};

mod assumption;
mod common;
mod field;
mod line;
mod reachability_graph;

type MultiSolution = HashSet<Field>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SolutionResult {
    Controversial,
    Unsolved,
    PartiallySolved(Field),
    Solved(MultiSolution)
}

pub use SolutionResult::*;

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

pub struct Solver {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
    max_depth: Option<usize>,
    find_all: bool
}

impl Solver {
    pub fn from_reader<R: io::Read>(rdr: R, max_depth: Option<usize>, find_all: bool) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self { row_hints: descr.row_hints, col_hints: descr.col_hints, max_depth, find_all } )
    }

    fn create_field(&self) -> Field {
        Field::new(self.nrows(), self.ncols())
    }

    fn nrows(&self) -> usize {
        self.row_hints.len()
    }

    fn ncols(&self) -> usize {
        self.col_hints.len()
    }

    fn row_line<'a>(&'a self, field: &'a mut Field, row_idx: usize) -> Line {
        Line {
            hints: &self.row_hints[row_idx],
            cells: field.row_mut(row_idx)
        }
    }

    fn col_line<'a>(&'a self, field: &'a mut Field, col_idx: usize) -> Line {
        Line {
            hints: &self.col_hints[col_idx],
            cells: field.col_mut(col_idx)
        }
    }

    /// Solves the nonogram only looking at a sinngle line at a time.
    ///
    /// The complete solution isn't guaranteed, the nonogram may be solved only partially.
    pub fn solve_by_lines(&self, field: &Field, line_cache: &mut LineCache) -> SolutionResult {
        let mut new_field = field.clone();
        for row_idx in 0..self.nrows() {
            let mut line = self.row_line(&mut new_field, row_idx);
            if line.solve(line_cache).is_none() {
                return Controversial;
            }
        }

        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.ncols());
        let mut changed_rows: HashSet<usize> = HashSet::with_capacity(self.nrows());
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                let mut line = self.col_line(&mut new_field, col_idx);
                match line.solve(line_cache) {
                    Some(ch) => changed_rows.extend(ch.iter()),
                    None => return Controversial
                }
            }
            if changed_rows.is_empty() {
                return if new_field.is_solved() {
                    Solved(HashSet::from([new_field]))
                } else if new_field != *field {
                    PartiallySolved(new_field)
                } else {
                    Unsolved
                }
            }

            changed_cols.clear();
            for &row_idx in changed_rows.iter() {
                let mut line = self.row_line(&mut new_field, row_idx);
                match line.solve(line_cache) {
                    Some(ch) => changed_cols.extend(ch.iter()),
                    None => return Controversial
                }
            }
            if changed_cols.is_empty() {
                return if new_field.is_solved() {
                    Solved(HashSet::from([new_field]))
                } else if new_field != *field {
                    PartiallySolved(new_field)
                } else {
                    Unsolved
                }
            }
        }
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.nrows()).cartesian_product(0..self.ncols())
    }

    fn iter_assumptions(&self) -> impl Iterator<Item = Assumption> {
        self.iter_coords().flat_map(|coords| KNOWN.iter().map(move |&val| Assumption {coords, val}))
    }

    fn do_solve(
        &self,
        field: &Field,
        depth: usize,
        assumptions: &Vec<Assumption>,
        line_cache: &mut LineCache
    ) -> SolutionResult {
        if self.max_depth.map(|d| depth > d).unwrap_or(false) {
            return Unsolved
        }
        let mut field = field.clone();
        let by_lines = self.solve_by_lines(&field, line_cache);
        match by_lines {
            Controversial | Solved(_) => return by_lines,
            PartiallySolved(new_field) => field.replace(new_field),
            Unsolved => ()
        };

        let mut solutions = HashSet::new();
        let mut new_assumptions = assumptions.clone();
        new_assumptions.push(Assumption::default());
        let mut prev_controversial: Option<Assumption> = None;
        let mut has_unsolved = false;
        let mut has_updates = false;
        for ass in self.iter_assumptions() {
            if field.get(ass.coords) != UNKNOWN {
                continue;
            }
            ass.apply(&mut field);
            new_assumptions[assumptions.len()] = ass.clone();
            match self.do_solve(&field, depth + 1, &new_assumptions, line_cache) {
                Solved(res) => {
                    solutions.extend(res);
                    if !self.find_all {
                        return Solved(solutions);
                    }
                    ass.unapply(&mut field);
                },
                Unsolved | PartiallySolved(_) => {
                    has_unsolved = true;
                    ass.unapply(&mut field);
                },
                Controversial => {
                    if let Some(prev_ass) = prev_controversial {
                        if prev_ass.coords == ass.coords {
                            return Controversial;
                        }
                    }
                    ass.invert().apply(&mut field);
                    match self.solve_by_lines(&field, line_cache) {
                        Controversial => return Controversial,
                        PartiallySolved(new_field) => field.replace(new_field),
                        Unsolved => (),
                        Solved(res) => {
                            solutions.extend(res);
                            if !self.find_all {
                                return Solved(solutions);
                            }
                        }
                    }
                    has_updates = true;
                    prev_controversial = Some(ass);
                }
            }
        }
        assert!(!field.is_solved());  // If it's solved, we should've caught it earlier
        if !solutions.is_empty() && !(has_unsolved && self.find_all) {
            Solved(solutions)
        } else if has_updates {
            PartiallySolved(field)
        } else {
            Unsolved
        }
    }

    pub fn solve(&self) -> SolutionResult {
        self.do_solve(&self.create_field(), 0, &Vec::new(), &mut HashMap::new())
    }

    pub fn solve_2sat(&self) -> SolutionResult {
        let mut field = self.create_field();
        let mut line_cache: LineCache = HashMap::new();
        let by_lines = self.solve_by_lines(&field, &mut line_cache);
        match by_lines {
            Controversial | Solved(_) => return by_lines,
            Unsolved => (),
            PartiallySolved(new_field) => field.replace(new_field),
        }
        println!("{}\n", field.to_string());

        let mut global_changed = false;
        loop {
            let mut reach: ReachabilityGraph<Assumption> = ReachabilityGraph::new();
            let old_field = field.clone();
            let mut solutions = HashSet::new();
            let mut has_unsolved = false;
            for ass1 in self.iter_assumptions() {
                if field.get(ass1.coords) != UNKNOWN {
                    continue;
                }
                for ass2 in self.iter_assumptions() {
                    if ass1.coords <= ass2.coords
                    || field.get(ass2.coords) != UNKNOWN
                    || reach.is_reachable(&ass1, &ass2.invert()) {
                        continue;
                    }
                    ass1.apply(&mut field);
                    ass2.apply(&mut field);
                    match self.solve_by_lines(&field, &mut line_cache) {
                        Unsolved | PartiallySolved(_) => has_unsolved = true,
                        Solved(res) => {
                            solutions.extend(res);
                            if !self.find_all { return Solved(solutions); }
                        },
                        Controversial => {
                            reach.set_reachable(&ass1, &ass2.invert());
                            reach.set_reachable(&ass2, &ass1.invert());
                        }
                    }
                    ass1.unapply(&mut field);
                    ass2.unapply(&mut field);
                }
            }

            if !has_unsolved {
                return Solved(solutions);
            }

            let mut changed = false;
            for ass in reach.get_impossible() {
                ass.invert().apply(&mut field);
                changed = true;
                global_changed = true;
            }
            if !changed {
                return if global_changed { PartiallySolved(field) } else { Unsolved }
            }

            let by_lines = self.solve_by_lines(&field, &mut line_cache);
            match by_lines {
                Solved(_) | Controversial => return by_lines,
                PartiallySolved(new_field) => field.replace(new_field),
                Unsolved => ()
            }
            println!("{}\n", field.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl SolutionResult {
        fn assert_solved(&self, results: &[&str]) {
            if let Solved(flds) = self {
                assert_eq!(
                    flds.iter().map(|f| f.to_string()).collect::<HashSet<String>>(),
                    results.iter().map(|x| x.to_string()).collect::<HashSet<String>>()
                );
            } else {
                panic!("Not solved: {self:?}");
            }
        }
    }

    #[test]
    fn solve_by_line() {
        let solver = Solver {
            row_hints: vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            col_hints: vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]],
            max_depth: None, find_all: false
        };
        solver.solve_by_lines(&solver.create_field(), &mut HashMap::new()).assert_solved(&["\
                *****\n\
                *XXXX\n\
                *****\n\
                XXXX*\n\
                *****\n\
        "]);
    }

    #[test]
    fn solve_ambiguous_recursive() {
        let solver = Solver {
            row_hints: vec![vec![1], vec![1]],
            col_hints: vec![vec![1], vec![1]],
            max_depth: Some(3), find_all: true
        };
        solver.solve().assert_solved(&["\
            *X\n\
            X*\n\
        ", "\
            X*\n\
            *X\n\
        "]);
    }

    #[test]
    fn solve_ambiguous_2sat() {
        let solver = Solver {
            row_hints: vec![vec![1], vec![1]],
            col_hints: vec![vec![1], vec![1]],
            max_depth: Some(3), find_all: true
        };
        solver.solve_2sat().assert_solved(&["\
            *X\n\
            X*\n\
        ", "\
            X*\n\
            *X\n\
        "]);
    }

    #[test]
    fn solve_double_ambiguous_recursive() {
        let solver = Solver {
            row_hints: vec![vec![1, 1], vec![1, 1]],
            col_hints: vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            max_depth: None, find_all: true
        };
        solver.solve().assert_solved(&["\
            *XX*X\n\
            X*XX*\n\
        ", "\
            *XXX*\n\
            X*X*X\n\
        ", "\
            X*XX*\n\
            *XX*X\n\
        ", "\
            X*X*X\n\
            *XXX*\n\
        "]);
    }

    #[ignore]
    #[test]
    fn solve_double_ambiguous_2sat() {
        let solver = Solver {
            row_hints: vec![vec![1, 1], vec![1, 1]],
            col_hints: vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            max_depth: None, find_all: true
        };
        solver.solve_2sat().assert_solved(&["\
            *XX*X\n\
            X*XX*\n\
        ", "\
            *XXX*\n\
            X*X*X\n\
        ", "\
            X*XX*\n\
            *XX*X\n\
        ", "\
            X*X*X\n\
            *XXX*\n\
        "]);
    }
}