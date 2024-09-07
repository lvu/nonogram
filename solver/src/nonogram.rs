use common::{LineHints, UNKNOWN};
use line::{Line, LineMut};
use ndarray::prelude::*;
use std::{collections::HashSet, io};

mod common;
mod line;

#[derive(Debug)]
pub struct Nonogram {
    field: Array2<i8>,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

impl Nonogram {
    pub fn from_reader<R: io::Read>(rdr: R) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self::from_hints(descr.row_hints, descr.col_hints))
    }

    fn from_hints(row_hints: Vec<LineHints>, col_hints: Vec<LineHints>) -> Self {
        Self {
            field: Array::from_elem((row_hints.len(), col_hints.len()), UNKNOWN),
            row_hints, col_hints
        }
    }
    
    fn row_line(&mut self, row_idx: usize) -> RowLine {
        RowLine {nono: self, row_idx}
    }
    
    fn col_line(&mut self, col_idx: usize) -> ColLine {
        ColLine {nono: self, col_idx}
    }
    
    fn solve_by_lines(&mut self) {
        for row_idx in 0..self.row_hints.len() {
            self.row_line(row_idx).solve();
        }
        
        let mut changed_cols: HashSet<usize> = HashSet::from_iter(0..self.col_hints.len());
        let mut changed_rows: HashSet<usize> = HashSet::new();
        loop {
            changed_rows.clear();
            for &col_idx in changed_cols.iter() {
                let ch = self.col_line(col_idx).solve();
                changed_rows.extend(ch.iter());
            }
            if changed_rows.is_empty() {
                break
            }

            changed_cols.clear();
            for &row_idx in changed_rows.iter() {
                let ch = self.row_line(row_idx).solve();
                changed_cols.extend(ch.iter());
            }
            if changed_cols.is_empty() {
                break
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

struct RowLine<'a> {
    nono: &'a mut Nonogram,
    row_idx: usize
}

impl<'a> Line for RowLine<'a> {
    fn hints(&self) -> &LineHints {
        &self.nono.row_hints[self.row_idx]
    }

    fn cells(&self) -> ArrayView1<i8> {
        self.nono.field.row(self.row_idx)
    }
}

impl<'a> LineMut for RowLine<'a> {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8> {
        self.nono.field.row_mut(self.row_idx)
    }
}

struct ColLine<'a> {
    nono: &'a mut Nonogram,
    col_idx: usize
}

impl<'a> Line for ColLine<'a> {
    fn hints(&self) -> &LineHints {
        &self.nono.col_hints[self.col_idx]
    }

    fn cells(&self) -> ArrayView1<i8> {
        self.nono.field.column(self.col_idx)
    }
}

impl<'a> LineMut for ColLine<'a> {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8> {
        self.nono.field.column_mut(self.col_idx)
    }
}

mod tests {
    use super::*;
    
    #[test]
    fn solve_by_line_simple() {
        let mut nono = Nonogram::from_hints(
            vec![vec![5], vec![1], vec![5], vec![1], vec![5]],
            vec![vec![3, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1], vec![1, 3]]
        );
        nono.solve_by_lines();
        let row_strs: Vec<String> = (0..nono.row_hints.len()).map(|idx| nono.row_line(idx).to_string()).collect();
        assert_eq!(row_strs, vec![
            "*****",
            "*XXXX",
            "*****",
            "XXXX*",
            "*****"
        ]);
    }


}