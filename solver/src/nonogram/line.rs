use super::assumption::Assumption;
use super::common::{line_to_str, CellValue, LineHints};
use crate::nonogram::common::KNOWN;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;
use CellValue::*;
use LineType::*;

#[cfg(test)]
mod tests;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum LineType {
    Row,
    Col,
}

pub struct Line<'a> {
    pub line_type: LineType,
    pub line_idx: usize,
    pub hints: &'a LineHints,
    pub cells: Cow<'a, [CellValue]>,
}

impl<'a> Line<'a> {
    pub fn new(line_type: LineType, line_idx: usize, hints: &'a LineHints, cells: &'a [CellValue]) -> Self {
        Self { line_type, line_idx, hints, cells: Cow::from(cells) }
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        line_to_str(&self.cells)
    }

    fn do_verify(&self, hint_idx: usize, cells_offset: usize) -> bool {
        if cells_offset >= self.cells.len() {
            return hint_idx == self.hints.len();
        }
        let cells = &self.cells[cells_offset..];
        if hint_idx == self.hints.len() {
            return cells.iter().all(|&x| x != Filled);
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
                && self.do_verify(hint_idx + 1, cells_offset + end + 1)
            {
                return true;
            }
            if val == Filled {
                return false;
            }
        }
        false
    }

    fn verify(&self) -> bool {
        self.do_verify(0, 0)
    }

    fn cache_key(&self) -> LineCacheKey {
        let mut packed_cells = vec![0u8; (self.cells.len() + 3) / 4];
        let mut idx = 0;
        for chunk in self.cells.chunks(4) {
            let c = match chunk {
                [b1, b2, b3, b4] => ((*b1 as u8) << 6) | ((*b2 as u8) << 4) | ((*b3 as u8) << 2) | (*b4 as u8),
                [b1, b2, b3] => ((*b1 as u8) << 4) | ((*b2 as u8) << 2) | (*b3 as u8),
                [b1, b2] => ((*b1 as u8) << 2) | (*b2 as u8),
                [b1] => *b1 as u8,
                _ => panic!("Impossible chunk: {chunk:?}")
            };
            packed_cells[idx] = c;
            idx += 1;
        }
        LineCacheKey { line_type: self.line_type, line_idx: self.line_idx, cells: packed_cells }
    }

    fn get_coords(&self, idx: usize) -> (usize, usize) {
        match self.line_type {
            Row => (self.line_idx, idx),
            Col => (idx, self.line_idx),
        }
    }

    fn do_solve(&mut self) -> Option<Vec<Assumption>> {
        if !self.verify() {
            return None;
        }
        let mut result = Vec::new();
        'idxs: for idx in 0..self.cells.len() {
            if self.cells[idx] != Unknown {
                continue;
            }

            for &val in KNOWN.iter() {
                self.cells.to_mut()[idx] = val;
                if !self.verify() {
                    let new_val = val.invert();
                    self.cells.to_mut()[idx] = new_val;
                    result.push(Assumption { coords: self.get_coords(idx), val: new_val });
                    continue 'idxs;
                }
            }

            self.cells.to_mut()[idx] = Unknown;
        }
        debug_assert!(self.verify());
        Some(result)
    }

    /// Solves the line to the extent currently possbile.
    ///
    /// Returns updates as a list of Assumption if the line wasn't controversial, None otherwise.
    pub fn solve<'b, S: BuildHasher>(&mut self, cache: &'b mut LineCache<S>) -> &'b Option<Vec<Assumption>> {
        let cache_key = self.cache_key();
        cache.entry(cache_key).or_insert_with(move || Box::new(self.do_solve()))
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct LineCacheKey {
    line_type: LineType,
    line_idx: usize,
    cells: Vec<u8>,
}

pub type LineCache<S> = HashMap<LineCacheKey, Box<Option<Vec<Assumption>>>, S>;
