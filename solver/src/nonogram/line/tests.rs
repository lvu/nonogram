use ndarray::Array1;

use super::*;

struct OwnedLine {
    hints: LineHints,
    cells: Array1<u8>,
}

impl OwnedLine {
    fn create(hints: LineHints, l: &str) -> Result<Self, std::fmt::Error> {
        let cells = l
            .chars()
            .map(|c| match c {
                '.' => Ok(UNKNOWN),
                '*' => Ok(FILLED),
                'X' => Ok(EMPTY),
                _ => Err(std::fmt::Error),
            })
            .collect::<Result<Array1<u8>, std::fmt::Error>>()?;
        Ok(Self { hints, cells })
    }

    fn line(&mut self) -> Line {
        Line { hints: &self.hints, cells: self.cells.view_mut() }
    }
}

#[test]
fn serialization_works() {
    let s = "*..X.**";
    let mut ol = OwnedLine::create(vec![2, 3], s).unwrap();
    assert_eq!(ol.line().to_string(), s);
}

#[test]
fn serialization_fails() {
    let s = "*..X.x**";
    assert!(OwnedLine::create(vec![2, 3], s).is_err());
}

#[test]
fn verify_plenty_space() {
    let mut ol = OwnedLine::create(vec![2, 3], "......").unwrap();
    assert!(ol.line().verify());
}

#[test]
fn verify_not_enough_space() {
    let mut ol = OwnedLine::create(vec![2, 3], ".....").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_separated_enough_space() {
    let mut ol = OwnedLine::create(vec![2, 3], "X..X.*.X").unwrap();
    assert!(ol.line().verify());
}

#[test]
fn verify_separated_not_enough_space() {
    let mut ol = OwnedLine::create(vec![2, 3], "X..X*.X").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_unsatisfialble_filled() {
    let mut ol = OwnedLine::create(vec![2, 3], "..*...").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_unsatisfialble_filled_with_frame() {
    let mut ol = OwnedLine::create(vec![2, 3], "X..*...X").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_split_with_badly_filled_left() {
    let mut ol = OwnedLine::create(vec![2, 3], "*..*X...").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_too_many_filled() {
    let mut ol = OwnedLine::create(vec![2, 3], "*..X.*.X*").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_split_with_fine_left() {
    let mut ol = OwnedLine::create(vec![2, 3], "*..*X..").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn solve_simple_overlap_and_unreachable() {
    let mut ol = OwnedLine::create(vec![4], ".....*..").unwrap();
    assert_eq!(ol.line().solve(&mut HashMap::new()), Some(HashSet::from([0, 1, 4])));
    assert_eq!(ol.line().to_string(), "XX..**..");
}

#[test]
fn solve_fill_with_ambiguity() {
    let mut ol = OwnedLine::create(vec![1, 2], "...*X..").unwrap();
    assert_eq!(ol.line().solve(&mut HashMap::new()), Some(HashSet::from([1])));
    assert_eq!(ol.line().to_string(), ".X.*X..");
}

#[test]
fn solve_empties_with_definite_chunks() {
    let mut ol = OwnedLine::create(vec![2, 1], "...X.*.X*").unwrap();
    assert_eq!(ol.line().solve(&mut HashMap::new()), Some(HashSet::from([0, 1, 2])));
    assert_eq!(ol.line().to_string(), "XXXX.*.X*");
}
