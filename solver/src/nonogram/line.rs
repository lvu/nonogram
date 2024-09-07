use std::collections::HashSet;

#[cfg(test)]
mod tests;

use ndarray::{s, ArrayView1, ArrayViewMut1};

use super::common::{LineHints, FILLED, UNKNOWN, EMPTY};

pub trait Line {
    fn hints(&self) -> &LineHints;
    fn cells(&self) -> ArrayView1<i8>;

    fn to_string(&self) -> String {
        self.cells().iter().map(|x| match *x {
            UNKNOWN => '.',
            FILLED => '*',
            EMPTY => 'X',
            _ => panic!("Invalid value: {x}")
        }).collect()
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
    /// Returns a set of indexes updated.
    /// 
    /// The line should be valid (`self.verify()` should be `true`) before calling.
    fn solve(&mut self) -> HashSet<usize> {
        debug_assert!(self.verify());
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
        result
    }
}