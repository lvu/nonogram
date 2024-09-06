use common::{LineHints, UNKNOWN};
use ndarray::prelude::*;
use std::io;

mod common;
mod line;

#[derive(Debug)]
pub struct Nonogram {
    field: Array2<i8>,
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}

struct FieldLine {
    hints: &Ve
}

impl Nonogram {
    pub fn from_reader<R: io::Read>(rdr: R) -> Result<Self, serde_json::Error> {
        let descr: NonoDescription = serde_json::from_reader(rdr)?;
        Ok(Self {
            field: Array::from_elem((descr.row_hints.len(), descr.col_hints.len()), UNKNOWN),
            row_hints: descr.row_hints,
            col_hints: descr.col_hints
        })
    }
}

#[derive(serde::Deserialize)]
struct NonoDescription {
    row_hints: Vec<LineHints>,
    col_hints: Vec<LineHints>
}