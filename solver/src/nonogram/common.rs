#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum CellValue {
    Filled,
    Empty,
    Unknown,
}

pub use CellValue::*;

pub const KNOWN: [CellValue; 2] = [Filled, Empty];

pub type LineHints = Vec<usize>;

pub fn line_to_str(line: &Vec<CellValue>) -> String {
    line.iter()
        .map(|x| match *x {
            Unknown => '.',
            Filled => '*',
            Empty => 'X',
        })
        .collect()
}

impl CellValue {
    pub fn invert(&self) -> Self {
        match self {
            Filled => Empty,
            Empty => Filled,
            _ => panic!("Cannot invert {self:?}"),
        }
    }
}

impl Default for CellValue {
    fn default() -> Self {
        Unknown
    }
}
