use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod tests;

use ndarray::{s, Array1, ArrayViewMut1};

use super::common::{line_to_str, LineHints, EMPTY, FILLED, UNKNOWN};

pub struct Line<'a> {
    pub hints: &'a LineHints,
    pub cells: ArrayViewMut1<'a, u8>
}

impl<'a> Line<'a> {
    fn to_string(&self) -> String {
        line_to_str(&self.cells)
    }

    fn do_verify(&self, hint_idx: usize, cells_offset: usize) -> bool {
        if cells_offset >= self.cells.len() {
            return hint_idx == self.hints.len();
        }
        let cells = self.cells.slice(s![cells_offset..]);
        if hint_idx == self.hints.len() {
            return cells.iter().all(|&x| x != FILLED);
        }
        let current_hint = self.hints[hint_idx];
        let size = cells.len();

        if current_hint > size {
            return false;
        }
        for (start, &val) in cells.slice(s![..size - current_hint + 1]).indexed_iter() {
            let end = start + current_hint;
            if cells.slice(s![start..end]).iter().all(|&x| x != EMPTY)
                && (end == size || cells[end] != FILLED)
                && self.do_verify(hint_idx + 1, cells_offset + end + 1)
            {
                return true;
            }
            if val == FILLED {
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
            acc = (acc << 2) | val;
            cnt += 1;
            if cnt == 4 {
                cells.push(acc);
                cnt = 0;
                acc = 0;
            }
        }
        cells.push(acc);
        LineCacheKey { hints: self.hints.clone(), cells }
    }

    /// Solves the line to the extent currently possbile, in-place.
    ///
    /// Returns a set of indexes updated if the line wasn't controversial, None therwise.
    pub fn solve(&mut self, cache: &mut LineCache) -> Option<HashSet<usize>> {
        let cache_key = self.cache_key();
        if let Some(cache_value) = cache.get(&cache_key) {
            match cache_value {
                Some((result, new_cells)) => {
                    if self.cells != new_cells {
                        self.cells.assign(new_cells);
                    }
                    return Some(result.clone());
                },
                None => return None
            }
        }
        if !self.verify() {
            cache.insert(cache_key, None);
            return None;
        }
        let mut result = HashSet::new();
        for idx in 0..self.cells.len() {
            if self.cells[idx] != UNKNOWN {
                continue;
            }

            self.cells[idx] = FILLED;
            if !self.verify() {
                self.cells[idx] = EMPTY;
                result.insert(idx);
                continue;
            }

            self.cells[idx] = EMPTY;
            if !self.verify() {
                self.cells[idx] = FILLED;
                result.insert(idx);
                continue;
            }

            self.cells[idx] = UNKNOWN;
        }
        debug_assert!(self.verify());
        cache.insert(cache_key, Some((result.clone(), self.cells.to_owned())));
        Some(result)
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct LineCacheKey {
    hints: LineHints,
    cells: Vec<u8>
}

pub type LineCache = HashMap<LineCacheKey, Option<(HashSet<usize>, Array1<u8>)>>;