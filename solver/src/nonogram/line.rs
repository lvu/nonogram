use std::collections::{HashMap, HashSet};

#[cfg(test)]
mod tests;

use ndarray::{s, Array1, ArrayView1, ArrayViewMut1};

use super::common::{line_to_str, LineHints, EMPTY, FILLED, UNKNOWN};

pub trait Line {
    fn hints(&self) -> &LineHints;
    fn cells(&self) -> ArrayView1<i8>;

    fn to_string(&self) -> String {
        line_to_str(self.cells())
    }

    fn do_verify(&self, hint_idx: usize, cells_offset: usize) -> bool {
        if cells_offset >= self.cells().len() {
            return hint_idx == self.hints().len();
        }
        let cells = self.cells();
        let cells = cells.slice(s![cells_offset..]);
        if hint_idx == self.hints().len() {
            return cells.iter().all(|&x| x != FILLED);
        }
        let current_hint = self.hints()[hint_idx];
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
}

pub trait LineMut: Line {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8>;

    /// Solves the line to the extent currently possbile, in-place.
    ///
    /// Returns a set of indexes updated if the line wasn't controversial, None therwise.
    fn solve(&mut self, cache: &mut SolveCache) -> Option<HashSet<usize>> {
        let cache_key = SolveCacheKey {
            hints: self.hints().clone(),
            cells: self.cells().clone().into_owned()
        };
        if let Some(cache_value) = cache.get(&cache_key) {
            unsafe {cache_hits += 1;}
            match cache_value {
                Some((result, new_cells)) => {
                    self.cells_mut().assign(new_cells);
                    return Some(result.clone());
                },
                None => return None
            }
        }
        unsafe {cache_misses += 1;}
        if !self.verify() {
            cache.insert(cache_key, None);
            return None;
        }
        let mut result = HashSet::new();
        for idx in 0..self.cells().len() {
            if self.cells()[idx] != UNKNOWN {
                continue;
            }

            self.cells_mut()[idx] = FILLED;
            if !self.verify() {
                self.cells_mut()[idx] = EMPTY;
                result.insert(idx);
                continue;
            }

            self.cells_mut()[idx] = EMPTY;
            if !self.verify() {
                self.cells_mut()[idx] = FILLED;
                result.insert(idx);
                continue;
            }

            self.cells_mut()[idx] = UNKNOWN;
        }
        debug_assert!(self.verify());
        cache.insert(cache_key, Some((result.clone(), self.cells().clone().to_owned())));
        Some(result)
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct SolveCacheKey {
    hints: LineHints,
    cells: Array1<i8>
}

pub type SolveCache = HashMap<SolveCacheKey, Option<(HashSet<usize>, Array1<i8>)>>;
pub static mut cache_hits: usize = 0;
pub static mut cache_misses: usize = 0;