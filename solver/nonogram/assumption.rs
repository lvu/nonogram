use super::common::{CellValue, Unknown};
use super::line::LineType;
use super::Field;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub struct Assumption {
    pub coords: (usize, usize),
    pub val: CellValue,
}

impl Assumption {
    pub fn invert(&self) -> Self {
        Self { coords: self.coords, val: self.val.invert() }
    }

    pub fn apply(&self, field: &mut Field) {
        field.set(self.coords, self.val);
    }

    pub fn unapply(&self, field: &mut Field) {
        field.set(self.coords, Unknown);
    }

    pub fn line_idx(&self, line_type: LineType) -> usize {
        match line_type {
            LineType::Row => self.coords.0,
            LineType::Col => self.coords.1,
        }
    }
}
