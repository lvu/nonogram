use ahash::AHasher;
use assumption::Assumption;
use clap::ValueEnum;
use common::{LineHints, Unknown, KNOWN};
use field::Field;
use itertools::Itertools;
use line::{Line, LineCache, LineType};
use reachability_graph::ReachabilityGraph;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::io;
use std::sync::{Arc, RwLock};
use LineType::*;

mod assumption;
mod common;
mod field;
mod line;
mod reachability_graph;

type MultiSolution = HashMap<Vec<u8>, Field>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SolutionResult {
    Controversial,
    Unsolved(Vec<Assumption>),
    Solved(MultiSolution),
}

#[derive(ValueEnum, Debug, Clone)]
pub enum Algorithm {
    ByLines,
    Naive,
    TwoSat,
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
    algorithm: Algorithm,
}

impl Solver {
    pub fn from_reader<R: io::Read>(
        rdr: R,
        max_depth: usize,
        find_all: bool,
        algorithm: Algorithm,
    ) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(
            descr.row_hints,
            descr.col_hints,
            max_depth,
            find_all,
            algorithm,
        ))
    }

    fn from_hints(
        row_hints: Vec<LineHints>,
        col_hints: Vec<LineHints>,
        max_depth: usize,
        find_all: bool,
        algorithm: Algorithm,
    ) -> Self {
        let row_cache = (0..row_hints.len())
            .map(|_| Arc::new(RwLock::new(HashMap::default())))
            .collect();
        let col_cache = (0..col_hints.len())
            .map(|_| Arc::new(RwLock::new(HashMap::default())))
            .collect();
        Self { row_hints, col_hints, row_cache, col_cache, max_depth, find_all, algorithm }
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

    fn cache(&self, line_type: LineType, line_idx: usize) -> LineCache<ABuildHasher> {
        match line_type {
            Row => self.row_cache[line_idx].clone(),
            Col => self.col_cache[line_idx].clone(),
        }
    }

    fn do_solve_by_lines_step(
        &self,
        field: &mut Cow<Field>,
        line_type: LineType,
        line_idxs: impl Iterator<Item = usize>,
    ) -> Option<Vec<Assumption>> {
        let mut all_changes: Vec<Assumption> = Vec::new();
        for line_idx in line_idxs {
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

    fn do_solve_by_lines(&self, field: &Field) -> SolutionResult {
        let mut field = Cow::Borrowed(field);
        let mut all_changes: Vec<Assumption> = Vec::new();
        match self.do_solve_by_lines_step(&mut field, Row, 0..self.nrows()) {
            None => return Controversial,
            Some(changes) => all_changes.extend(changes),
        }
        let mut line_type = Col;
        let mut changed_idxs: HashSet<usize> = (0..self.ncols()).collect();
        loop {
            match self.do_solve_by_lines_step(&mut field, line_type, changed_idxs.into_iter()) {
                None => return Controversial,
                Some(changes) => {
                    if changes.is_empty() {
                        return if field.is_solved() {
                            Solved(HashMap::from([(field.key(), field.into_owned())]))
                        } else {
                            Unsolved(all_changes)
                        };
                    }
                    changed_idxs = changes.iter().map(|ass| ass.line_idx(line_type.other())).collect();
                    all_changes.extend(changes);
                }
            }
            line_type = line_type.other();
        }
    }

    fn iter_coords(&self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.nrows()).cartesian_product(0..self.ncols())
    }

    fn iter_assumptions(&self) -> impl Iterator<Item = Assumption> {
        self.iter_coords()
            .flat_map(|coords| KNOWN.iter().map(move |&val| Assumption { coords, val }))
    }

    fn do_step(&self, field: &Field, depth: usize) -> SolutionResult
    {
        match self.algorithm {
            Algorithm::Naive => self.do_naive_step(field, depth),
            Algorithm::TwoSat => self.do_2sat_step(field, depth),
            Algorithm::ByLines => panic!("ByLines shouldn't get here"),
        }
    }

    fn do_naive_step(&self, field: &Field, max_depth: usize) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();
        let mut solutions = HashMap::new();
        let mut has_unsolved = false;
        for coords in self.iter_coords() {
            if field.get(coords) != Unknown {
                continue;
            }
            let mut has_controversy = false;
            for val in KNOWN {
                let ass = Assumption { coords, val };
                ass.apply(&mut field);
                match self.do_solve(&field, max_depth) {
                    Solved(res) => {
                        extend_solutions_from(&mut solutions, res);
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
        Unsolved(all_changes)
    }

    fn do_2sat_step(
        &self,
        field: &Field,
        max_depth: usize,
    ) -> SolutionResult {
        let mut field = field.clone();
        let mut reach = ReachabilityGraph::new();
        let mut solutions = HashMap::new();
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
                match self.do_solve(&field, max_depth) {
                    Unsolved(_) => has_unsolved = true,
                    Solved(res) => {
                        extend_solutions_from(&mut solutions, res);
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

        if solutions.len() > 0 && !(has_unsolved && self.find_all) {
            return Solved(solutions);
        }

        self.apply_impossible_matches(&field, &reach)
    }

    fn do_solve(&self, field: &Field, max_depth: usize) -> SolutionResult {
        let mut field = field.clone();
        let mut all_changes = Vec::new();

        'outer: loop {
            let by_lines = self.do_solve_by_lines(&field);
            match by_lines {
                Controversial | Solved(_) => return by_lines,
                Unsolved(changes) => {
                    if max_depth == 0 {
                        return Unsolved(changes);
                    }
                    apply_changes(&changes, &mut field, &mut all_changes);
                }
            }

            for depth in 0..max_depth {
                let by_step = self.do_step(&field, depth);
                match by_step {
                    Solved(_) | Controversial => return by_step,
                    Unsolved(changes) => {
                        if !changes.is_empty() {
                            apply_changes(&changes, &mut field, &mut all_changes);
                            continue 'outer;
                        }
                    }
                }
            }
            return Unsolved(all_changes)
        }
    }

    pub fn solve(&self) -> SolutionResult {
        let field = self.create_field();
        match self.algorithm {
            Algorithm::ByLines => self.do_solve_by_lines(&field),
            _ => self.do_solve(&field, self.max_depth),
        }
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
    use rstest::rstest;

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
            Algorithm::ByLines,
        );
        solver.solve().assert_solved(&["\
                #####\n\
                #....\n\
                #####\n\
                ....#\n\
                #####\n\
        "]);
    }

    #[rstest]
    #[case(Algorithm::TwoSat)]
    #[case(Algorithm::Naive)]
    fn solve_ambiguous(#[case] algorithm: Algorithm) {
        let solver = Solver::from_hints(vec![vec![1], vec![1]], vec![vec![1], vec![1]], 3, true, algorithm);
        solver.solve().assert_solved(&[
            "#.\n\
             .#\n",
            ".#\n\
             #.\n",
        ]);
    }


    #[rstest]
    #[case(Algorithm::TwoSat)]
    #[case(Algorithm::Naive)]
    fn solve_double_ambiguous_naive(#[case] algorithm: Algorithm) {
        let solver = Solver::from_hints(
            vec![vec![1, 1], vec![1, 1]],
            vec![vec![1], vec![1], vec![], vec![1], vec![1]],
            2,
            true,
            algorithm
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
