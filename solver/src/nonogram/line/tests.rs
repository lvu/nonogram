use ndarray::Array1;

use super::*;

struct OwnedLine {
    hints: LineHints,
    cells: Array1<i8>
}

impl OwnedLine {
    fn create(hints: LineHints, l: &str) -> Result<Self, std::fmt::Error> {
        let cells = l.chars().map(|c| match c {
            '.' => Ok(UNKNOWN),
            '*' => Ok(FILLED),
            'X' => Ok(EMPTY),
            _ => Err(std::fmt::Error)
        }).collect::<Result<Array1<i8>, std::fmt::Error>>()?;
        Ok(Self { hints, cells })
    }
}

impl Line for OwnedLine {
    fn hints(&self) -> &LineHints {
        &self.hints
    }

    fn cells(&self) -> ArrayView1<i8> {
        self.cells.view()
    }
}

impl LineMut for OwnedLine {
    fn cells_mut(&mut self) -> ArrayViewMut1<i8> {
        self.cells.view_mut()
    }
}

#[test]
fn serialization_works() {
    let s = "*..X.**";
    let ol = OwnedLine::create(vec![2, 3], s).unwrap();
    assert_eq!(ol.to_string(), s);
}

#[test]
fn serialization_fails() {
    let s = "*..X.x**";
    assert!(OwnedLine::create(vec![2, 3], s).is_err());
}

#[test]
fn verify_plenty_space() {
    let ol = OwnedLine::create(vec![2, 3], "......").unwrap();
    assert!(ol.verify());
}

#[test]
fn verify_not_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], ".....").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_separated_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], "X..X.*.X").unwrap();
    assert!(ol.verify());
}

#[test]
fn verify_separated_not_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], "X..X*.X").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_unsatisfialble_filled() {
    let ol = OwnedLine::create(vec![2, 3], "..*...").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_unsatisfialble_filled_with_frame() {
    let ol = OwnedLine::create(vec![2, 3], "X..*...X").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_split_with_badly_filled_left() {
    let ol = OwnedLine::create(vec![2, 3], "*..*X...").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_too_many_filled() {
    let ol = OwnedLine::create(vec![2, 3], "*..X.*.X*").unwrap();
    assert!(!ol.verify());
}

#[test]
fn verify_split_with_fine_left() {
    let ol = OwnedLine::create(vec![2, 3], "*..*X..").unwrap();
    assert!(!ol.verify());
}

#[test]
fn solve_simple_overlap_and_unreachable() {
    let mut ol = OwnedLine::create(vec![4], ".....*..").unwrap();
    ol.solve();
    assert_eq!(ol.to_string(), "XX..**..");
}

#[test]
fn solve_fill_with_ambiguity() {
    let mut ol = OwnedLine::create(vec![1, 2], "...*X..").unwrap();
    ol.solve();
    assert_eq!(ol.to_string(), ".X.*X..");
}

#[test]
fn solve_empties_with_definite_chunks() {
    let mut ol = OwnedLine::create(vec![2, 1], "...X.*.X*").unwrap();
    ol.solve();
    assert_eq!(ol.to_string(), "XXXX.*.X*");
}
