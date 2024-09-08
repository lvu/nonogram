use ndarray::ArrayView1;

pub const FILLED: i8 = 1;
pub const EMPTY: i8 = -1;
pub const UNKNOWN: i8 = 0;

pub type LineHints = Vec<usize>;

pub fn line_to_str(line: ArrayView1<i8>) -> String {
    line.iter().map(|x| match *x {
        UNKNOWN => '.',
        FILLED => '*',
        EMPTY => 'X',
        _ => panic!("Invalid value: {x}")
    }).collect()
}