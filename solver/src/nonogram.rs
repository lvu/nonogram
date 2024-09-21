use ahash::AHasher;
use assumption::Assumption;
use common::{LineHints, Unknown, KNOWN};
use field::Field;
use itertools::Itertools;
use line::{Line, LineCache, LineType};
use reachability_graph::ReachabilityGraph;
use std::borrow::{BorrowMut, Cow};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::io;
use std::ops::DerefMut;
use LineType::*;

mod assumption;
mod common;
mod field;
mod line;
mod reachability_graph;

type MultiSolution = HashSet<Field>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SolutionResult {
    Controversial,
    Unsolved(Vec<Assumption>),
    Solved(MultiSolution),
}

pub use SolutionResult::*;

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
}

pub struct Solver {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>,
    max_depth: Option<usize>,
    find_all: bool,
    line_cache: RefCell<LineCache<BuildHasherDefault<AHasher>>>,
}

impl Solver {
    pub fn from_reader<R: io::Read>(
        rdr: R,
        max_depth: Option<usize>,
        find_all: bool,
    ) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(descr.row_hints, descr.col_hints, max_depth, find_all))
    }

    fn from_hints(
        row_hints: Vec<LineHints>,
        col_hints: Vec<LineHints>,
        max_depth: Option<usize>,
        find_all: bool,
    ) -> Self {
        Self { row_hints, col_hints, max_depth, find_all, line_cache: RefCell::new(HashMap::default()) }
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

    fn do_solve_by_lines(&self, field: &Field) -> SolutionResult {
        let mut field = Cow::Borrowed(field);
        let mut all_changes: Vec<Assumption> = Vec::new();
        for row_idx in 0..self.nrows() {
            let mut line = self.row_line(&field, row_idx);
            match line.solve(self.line_cache.borrow_mut().deref_mut()) {
                Some(changes) if !changes.is_empty() => {
                    apply_changes(changes, field.to_mut(), &mut all_changes);
                }
                None => return Controversial,
                _ => ()
            }
        }

        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.ncols());
        let mut changed_rows: HashSet<usize> = HashSet::with_capacity(self.nrows());
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                let mut line = self.col_line(&field, col_idx);
                match line.solve(self.line_cache.borrow_mut().deref_mut()) {
                    Some(changes) if !changes.is_empty() => {
                        apply_changes(changes, field.to_mut(), &mut all_changes);
                        for ass in changes {
                            changed_rows.insert(ass.coords.0);
                        }
                    }
                    None => return Controversial,
                    _ => ()
                }
            }
            if field.is_solved() {
                return Solved(HashSet::from([field.into_owned()]));
            }
            if changed_rows.is_empty() {
                return Unsolved(all_changes);
            }

            changed_cols.clear();
            for &row_idx in changed_rows.iter() {
                let mut line = self.row_line(&field, row_idx);
                match line.solve(self.line_cache.borrow_mut().deref_mut()) {
                    Some(changes) if !changes.is_empty() => {
                        apply_changes(changes, field.to_mut(), &mut all_changes);
                        for ass in changes {
                            changed_cols.insert(ass.coords.1);
                        }
                    }
                    None => return Controversial,
                    _ => ()
                }
            }
            if field.is_solved() {
                return Solved(HashSet::from([field.into_owned()]));
            }
            if changed_cols.is_empty() {
                return Unsolved(all_changes);
            }
        }
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.nrows()).cartesian_product(0..self.ncols())
    }

    fn iter_assumptions(&self) -> impl Iterator<Item = Assumption> {
        self.iter_coords()
            .flat_map(|coords| KNOWN.iter().map(move |&val| Assumption { coords, val }))
    }

    fn max_depth_reached(&self, depth: usize) -> bool {
        self.max_depth.map(|d| depth > d).unwrap_or(false)
    }

    fn do_solve(&self, field: &Field, depth: usize) -> SolutionResult {
        if self.max_depth_reached(depth) {
            return Unsolved(Vec::new());
        }
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        let by_lines = self.do_solve_by_lines(&field);
        match by_lines {
            Controversial | Solved(_) => return by_lines,
            Unsolved(changes) => apply_changes(&changes, &mut field, &mut all_changes),
        };

        let mut solutions = HashSet::new();
        let mut has_unsolved = false;
        for coords in self.iter_coords() {
            if field.get(coords) != Unknown {
                continue;
            }
            let mut has_controversy = false;
            for val in KNOWN {
                let ass = Assumption { coords, val };
                ass.apply(&mut field);
                match self.do_solve(&field, depth + 1) {
                    Solved(res) => {
                        solutions.extend(res);
                        if !self.find_all {
                            return Solved(solutions);
                        }
                        ass.unapply(&mut field);
                    }
                    Unsolved(_) => {
                        has_unsolved = true;
                        ass.unapply(&mut field);
                    }
                    Controversial => {
                        if has_controversy {
                            return Controversial;
                        }
                        ass.invert().apply(&mut field);
                        match self.do_solve_by_lines(&field) {
                            Controversial => return Controversial,
                            Unsolved(changes) => apply_changes(&changes, &mut field, &mut all_changes),
                            Solved(res) => {
                                solutions.extend(res);
                                if !self.find_all {
                                    return Solved(solutions);
                                }
                            }
                        }
                        has_controversy = true;
                    }
                }
            }
        }
        if !solutions.is_empty() && !(has_unsolved && self.find_all) {
            return Solved(solutions);
        }
        match self.do_solve_by_lines(&field) {
            Solved(res) => {
                assert_eq!(solutions, res);
                Solved(res)
            }
            Unsolved(changes) => {
                all_changes.extend(changes);
                Unsolved(all_changes)
            }
            Controversial => Controversial,
        }
    }

    fn apply_impossible_matches(&self, field: &Field, reach: &ReachabilityGraph<Assumption>) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        for ass in reach.get_impossible() {
            let old_val = field.get(ass.coords);
            if old_val == Unknown {
                let ass_inv = ass.invert();
                ass_inv.apply(&mut field);
                all_changes.push(ass_inv);
            } else if old_val == ass.val {
                return Controversial;
            }
        }
        if all_changes.is_empty() {
            return Unsolved(all_changes);
        }
        let by_lines = self.do_solve_by_lines(&field);
        match by_lines {
            Unsolved(changes) => {
                all_changes.extend_from_slice(&changes);
                Unsolved(all_changes)
            }
            _ => by_lines,
        }
    }

    fn do_2sat_step<F: Fn(&Field, usize) -> SolutionResult>(
        &self,
        field: &Field,
        depth: usize,
        recurse: F,
    ) -> SolutionResult {
        let mut field = field.clone();
        let mut reach = ReachabilityGraph::new();
        let mut solutions = HashSet::new();
        let mut has_unsolved = false;
        for ass1 in self.iter_assumptions() {
            if field.get(ass1.coords) != Unknown {
                continue;
            }
            ass1.apply(&mut field);
            for ass2 in self.iter_assumptions() {
                if ass1.coords <= ass2.coords
                    || field.get(ass2.coords) != Unknown
                    || reach.is_reachable(&ass1, &ass2.invert())
                {
                    continue;
                }
                ass2.apply(&mut field);
                match recurse(&field, depth + 1) {
                    Unsolved(_) => has_unsolved = true,
                    Solved(res) => {
                        solutions.extend(res);
                        if !self.find_all {
                            return Solved(solutions);
                        }
                    }
                    Controversial => {
                        reach.set_reachable(&ass1, &ass2.invert());
                        reach.set_reachable(&ass2, &ass1.invert());
                    }
                }
                ass2.unapply(&mut field);
            }
            ass1.unapply(&mut field);
        }

        if solutions.len() > 0 && !has_unsolved {
            return Solved(solutions);
        }

        self.apply_impossible_matches(&field, &reach)
    }

    fn do_solve_2sat(&self, field: &Field, depth: usize) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        let by_lines = self.do_solve_by_lines(&field);
        match by_lines {
            Controversial | Solved(_) => return by_lines,
            Unsolved(changes) => {
                if self.max_depth_reached(depth) {
                    return Unsolved(changes);
                }
                apply_changes(&changes, &mut field, &mut all_changes);
            }
        }
        if depth == 0 {
            println!("{}\n", field.to_string());
        }

        loop {
            let by_step = self.do_2sat_step(&field, depth, |fld, _| self.do_solve_by_lines(fld));
            match by_step {
                Solved(_) | Controversial => return by_step,
                Unsolved(changes) => {
                    if changes.is_empty() {
                        break;
                    }
                    apply_changes(&changes, &mut field, &mut all_changes);
                }
            }
            if depth == 0 {
                println!("{}\n", field.to_string());
            }
        }
        loop {
            let by_step = self.do_2sat_step(&field, depth, |fld, d| self.do_solve_2sat(fld, d + 1));
            match by_step {
                Solved(_) | Controversial => return by_step,
                Unsolved(changes) => {
                    if changes.is_empty() {
                        break;
                    }
                    apply_changes(&changes, &mut field, &mut all_changes);
                }
            }
            if depth == 0 {
                println!("{}\n", field.to_string());
            }
        }
        Unsolved(all_changes)
    }

    pub fn solve_by_lines(&self) -> SolutionResult {
        self.do_solve_by_lines(&self.create_field())
    }

    pub fn solve(&self) -> SolutionResult {
        self.do_solve(&self.create_field(), 0)
    }

    pub fn solve_2sat(&self) -> SolutionResult {
        self.do_solve_2sat(&self.create_field(), 0)
    }
}

fn apply_changes(changes: &[Assumption], field: &mut Field, all_changes: &mut Vec<Assumption>) {
    all_changes.extend_from_slice(&changes);
    changes.iter().for_each(|ass| ass.apply(field));
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
        let solver = Solver::from_hints(
            vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]],
            None,
            false,
        );
        solver.do_solve_by_lines(&solver.create_field()).assert_solved(&["\
                *****\n\
                *XXXX\n\
                *****\n\
                XXXX*\n\
                *****\n\
        "]);
    }

    #[test]
    fn solve_ambiguous_recursive() {
        let solver = Solver::from_hints(vec![vec![1], vec![1]], vec![vec![1], vec![1]], Some(3), true);
        solver.solve().assert_solved(&[
            "*X\n\
             X*\n",
            "X*\n\
             *X\n",
        ]);
    }

    #[test]
    fn solve_ambiguous_2sat() {
        let solver = Solver::from_hints(vec![vec![1], vec![1]], vec![vec![1], vec![1]], Some(3), true);
        solver.solve_2sat().assert_solved(&[
            "*X\n\
             X*\n",
            "X*\n\
             *X\n",
        ]);
    }

    #[test]
    fn solve_double_ambiguous_recursive() {
        let solver = Solver::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            Some(2),
            true,
        );
        solver.solve().assert_solved(&[
            "*XX*X\n\
             X*XX*\n",
            "*XXX*\n\
             X*X*X\n",
            "X*XX*\n\
             *XX*X\n",
            "X*X*X\n\
             *XXX*\n",
        ]);
    }

    #[test]
    fn solve_double_ambiguous_2sat() {
        let solver = Solver::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            Some(2),
            true,
        );
        solver.solve_2sat().assert_solved(&[
            "*XX*X\n\
             X*XX*\n",
            "*XXX*\n\
             X*X*X\n",
            "X*XX*\n\
             *XX*X\n",
            "X*X*X\n\
             *XXX*\n",
        ]);
    }
}
