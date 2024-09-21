use std::collections::HashMap;
use crate::nonogram::common::KNOWN;
use super::assumption::Assumption;
use super::common::{line_to_str, CellValue, LineHints};
use CellValue::*;
use LineType::*;

#[cfg(test)]
mod tests;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum LineType {Row, Col}

pub struct Line<'a> {
    pub line_type: LineType,
    pub line_idx: usize,
    pub hints: &'a LineHints,
    pub cells: Vec<CellValue>,
}

impl<'a> Line<'a> {
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
        let mut cells = Vec::with_capacity(self.cells.len() / 4 + 1);
        let mut cnt: u8 = 0;
        let mut acc: u8 = 0;
        for &val in self.cells.iter() {
            acc = (acc << 2) | (val as u8);
            cnt += 1;
            if cnt == 4 {
                cells.push(acc);
                cnt = 0;
                acc = 0;
            }
        }
        cells.push(acc);
        LineCacheKey { line_type: self.line_type, line_idx: self.line_idx, cells }
    }

    fn get_coords(&self, idx: usize) -> (usize, usize) {
        match self.line_type {
            Row => (self.line_idx, idx),
            Col => (idx, self.line_idx),
        }
    }

    /// Solves the line to the extent currently possbile, in-place.
    ///
    /// Returns a set of indexes updated if the line wasn't controversial, None therwise.
    pub fn solve(mut self, cache: &mut LineCache) -> Option<Vec<Assumption>> {
        let cache_key = self.cache_key();
        if let Some(cache_value) = cache.get(&cache_key) {
            return cache_value.clone();
        }
        if !self.verify() {
            cache.insert(cache_key, None);
            return None;
        }
        let mut result = Vec::new();
        'idxs: for idx in 0..self.cells.len() {
            if self.cells[idx] != Unknown {
                continue;
            }

            for &val in KNOWN.iter() {
                self.cells[idx] = val;
                if !self.verify() {
                    let new_val = val.invert();
                    self.cells[idx] = new_val;
                    result.push(Assumption {coords: self.get_coords(idx), val: new_val});
                    continue 'idxs;
                }
            }

            self.cells[idx] = Unknown;
        }
        debug_assert!(self.verify());
        cache.insert(cache_key, Some(result.clone()));
        Some(result)
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct LineCacheKey {
    line_type: LineType,
    line_idx: usize,
    cells: Vec<u8>,
}

pub type LineCache = HashMap<LineCacheKey, Option<Vec<Assumption>>>;
