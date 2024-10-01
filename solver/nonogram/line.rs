use super::assumption::Assumption;
use super::common::{line_to_str, CellValue, LineHints};
use crate::nonogram::common::KNOWN;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::rc::Rc;
use CellValue::*;
use LineType::*;

#[cfg(test)]
mod tests;

pub type LineCache<S> = RefCell<HashMap<Vec<CellValue>, LineSolution, S>>;
pub type LineSolution = Rc<Option<Vec<Assumption>>>;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum LineType {
    Row,
    Col,
}

impl LineType {
    pub fn other(&self) -> Self {
        match *self {
            Row => Col,
            Col => Row,
        }
    }
}

pub struct Line<'a> {
    line_type: LineType,
    line_idx: usize,
    hints: &'a LineHints,
    cells: Cow<'a, [CellValue]>,
}

impl<'a> Line<'a> {
    pub fn new(line_type: LineType, line_idx: usize, hints: &'a LineHints, cells: &'a [CellValue]) -> Self {
        Self { line_type, line_idx, hints, cells: Cow::from(cells) }
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        line_to_str(&self.cells)
    }

    fn do_verify(&self, hint_idx: usize, cells_offset: usize, last_filled: Option<usize>) -> bool {
        if cells_offset >= self.cells.len() {
            return hint_idx == self.hints.len();
        }
        let cells = &self.cells[cells_offset..];
        if hint_idx == self.hints.len() {
            return last_filled.map_or(true, |lf| cells_offset > lf);
        }
        let current_hint = self.hints[hint_idx];
        let size = cells.len();

        if current_hint > size {
            return false;
        }
        for (start, &val) in cells[..size - current_hint + 1].iter().enumerate() {
            let end = start + current_hint;
            if cells[start..end].iter().all(|&x| x != Empty)
                && (end == size || cells[end] != Filled)
                && self.do_verify(hint_idx + 1, cells_offset + end + 1, last_filled)
            {
                return true;
            }
            if val == Filled {
                return false;
            }
        }
        false
    }

    fn verify(&self, last_filled: Option<usize>) -> bool {
        self.do_verify(0, 0, last_filled)
    }

    fn get_coords(&self, idx: usize) -> (usize, usize) {
        match self.line_type {
            Row => (self.line_idx, idx),
            Col => (idx, self.line_idx),
        }
    }

    fn get_last_filled(&self) -> Option<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == Filled)
            .map(|(idx, _)| idx)
            .last()
    }

    fn do_solve(&mut self) -> Option<Vec<Assumption>> {
        let mut last_filled = self.get_last_filled();
        if !self.verify(last_filled) {
            return None;
        }
        let mut result = Vec::new();
        'idxs: for idx in 0..self.cells.len() {
            if self.cells[idx] != Unknown {
                continue;
            }

            for &val in KNOWN.iter() {
                self.cells.to_mut()[idx] = val;
                if !self.verify(match val {
                    Filled => Some(opt_max(last_filled, idx)),
                    _ => last_filled,
                }) {
                    let new_val = val.invert();
                    self.cells.to_mut()[idx] = new_val;
                    result.push(Assumption { coords: self.get_coords(idx), val: new_val });
                    if new_val == Filled {
                        last_filled = Some(opt_max(last_filled, idx));
                    }
                    continue 'idxs;
                }
            }

            self.cells.to_mut()[idx] = Unknown;
        }
        debug_assert!(self.verify(last_filled));
        Some(result)
    }

    /// Solves the line to the extent currently possbile.
    ///
    /// Returns updates as a list of Assumption if the line wasn't controversial, None otherwise.
    pub fn solve<S>(&mut self, cache: &LineCache<S>) -> LineSolution
    where
        S: BuildHasher,
    {
        let entry = cache.borrow().get(self.cells.as_ref()).map(|x| x.clone());
        match entry {
            Some(result) => result.clone(),
            None => {
                let key = Vec::from(self.cells.as_ref());
                let result = self.do_solve();
                cache.borrow_mut().entry(key).or_insert(Rc::new(result)).clone()
            }
        }
    }
}

fn opt_max<T: Ord + Copy>(a: Option<T>, b: T) -> T {
    a.map_or(b, |v| v.max(b))
}
