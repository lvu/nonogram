use ndarray::ArrayView1;

pub const FILLED: u8 = 1;
pub const EMPTY: u8 = 2;
pub const UNKNOWN: u8 = 0;
pub const KNOWN: [u8; 2] = [FILLED, EMPTY];

pub type LineHints = Vec<usize>;

pub fn line_to_str(line: ArrayView1<u8>) -> String {
    line.iter().map(|x| match *x {
        UNKNOWN => '.',
        FILLED => '*',
        EMPTY => 'X',
        _ => panic!("Invalid value: {x}")
    }).collect()
}

pub fn invert_value(val: u8) -> u8 {
    match val {
        FILLED => EMPTY,
        EMPTY => FILLED,
        _ => panic!("Invalid value {val}")
    }
}