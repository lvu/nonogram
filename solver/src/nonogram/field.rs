use std::fmt::Display;

use ndarray::{Array, Array2, ArrayViewMut1};

use super::common::{line_to_str, CellValue, Unknown};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Field {
    data: Array2<CellValue>,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.rows() {
            writeln!(f, "{}", line_to_str(&row))?;
        }
        Ok(())
    }
}

impl Field {
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self { data: Array::from_elem((nrows, ncols), Unknown) }
    }

    pub fn is_solved(&self) -> bool {
        self.data.iter().all(|&x| x != Unknown)
    }

    pub fn row_mut(&mut self, idx: usize) -> ArrayViewMut1<CellValue> {
        self.data.row_mut(idx)
    }

    pub fn col_mut(&mut self, idx: usize) -> ArrayViewMut1<CellValue> {
        self.data.column_mut(idx)
    }

    pub fn get(&self, coords: (usize, usize)) -> CellValue {
        self.data[coords]
    }

    pub fn set(&mut self, coords: (usize, usize), val: CellValue) {
        self.data[coords] = val;
    }

    pub fn replace(&mut self, other: Self) {
        other.data.move_into(&mut self.data);
    }
}
