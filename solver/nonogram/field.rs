use super::common::{line_to_str, CellValue, Unknown};
use std::fmt::Display;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Field {
    nrows: usize,
    ncols: usize,
    rows: Vec<CellValue>,
    cols: Vec<CellValue>,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row_idx in 0..self.nrows {
            writeln!(f, "{}", line_to_str(self.row(row_idx)))?;

        }
        Ok(())
    }
}

impl Field {
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            rows: vec![Unknown; nrows * ncols],
            cols: vec![Unknown; nrows * ncols],
        }
    }

    pub fn is_solved(&self) -> bool {
        self.rows.iter().all(|&x| x != Unknown)
    }

    pub fn row(&self, idx: usize) -> &[CellValue] {
        &self.rows[idx * self.ncols .. (idx + 1) * self.ncols]
    }

    pub fn col(&self, idx: usize) -> &[CellValue] {
        &self.cols[idx * self.nrows .. (idx + 1) * self.nrows]
    }

    pub fn get(&self, coords: (usize, usize)) -> CellValue {
        let (row_idx, col_idx) = coords;
        self.rows[row_idx * self.ncols + col_idx]
    }

    pub fn set(&mut self, coords: (usize, usize), val: CellValue) {
        let (row_idx, col_idx) = coords;
        self.rows[row_idx * self.ncols + col_idx] = val;
        self.cols[col_idx * self.nrows + row_idx] = val;
    }

    pub fn key(&self) -> Vec<CellValue> {
        self.rows.clone()
    }
}
