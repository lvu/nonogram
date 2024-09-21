use super::common::{line_to_str, CellValue, Unknown};
use std::fmt::Display;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Field {
    rows: Vec<Vec<CellValue>>,
    cols: Vec<Vec<CellValue>>,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter() {
            writeln!(f, "{}", line_to_str(row))?;
        }
        Ok(())
    }
}

impl Field {
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            rows: (0..nrows).map(|_| vec![Unknown; ncols]).collect(),
            cols: (0..ncols).map(|_| vec![Unknown; nrows]).collect(),
        }
    }

    pub fn is_solved(&self) -> bool {
        self.rows.iter().all(|row| row.iter().all(|&x| x != Unknown))
    }

    pub fn row(&self, idx: usize) -> &[CellValue] {
        &self.rows[idx]
    }

    pub fn col(&self, idx: usize) -> &[CellValue] {
        &self.cols[idx]
    }

    pub fn get(&self, coords: (usize, usize)) -> CellValue {
        let (row_idx, col_idx) = coords;
        self.rows[row_idx][col_idx]
    }

    pub fn set(&mut self, coords: (usize, usize), val: CellValue) {
        let (row_idx, col_idx) = coords;
        self.rows[row_idx][col_idx] = val;
        self.cols[col_idx][row_idx] = val;
    }

    pub fn replace(&mut self, mut other: Self) {
        std::mem::swap(&mut self.rows, &mut other.rows);
        std::mem::swap(&mut self.cols, &mut other.cols);
    }
}