use std::collections::HashSet;

use super::*;

struct OwnedLine {
    hints: LineHints,
    cells: Vec<CellValue>,
}

impl OwnedLine {
    fn create(hints: LineHints, l: &str) -> Result<Self, std::fmt::Error> {
        let cells = l
            .chars()
            .map(|c| match c {
                '~' => Ok(Unknown),
                '#' => Ok(Filled),
                '.' => Ok(Empty),
                _ => Err(std::fmt::Error),
            })
            .collect::<Result<Vec<CellValue>, std::fmt::Error>>()?;
        Ok(Self { hints, cells })
    }

    fn line(&self) -> Line {
        Line::new(Row, 0, &self.hints, &self.cells)
    }
}

#[test]
fn serialization_works() {
    let s = "#~~.~##";
    let ol = OwnedLine::create(vec![2, 3], s).unwrap();
    assert_eq!(ol.line().to_string(), s);
}

#[test]
fn serialization_fails() {
    let s = "#~~.~,##";
    assert!(OwnedLine::create(vec![2, 3], s).is_err());
}

#[test]
fn verify_plenty_space() {
    let ol = OwnedLine::create(vec![2, 3], "~~~~~~").unwrap();
    assert!(ol.line().verify());
}

#[test]
fn verify_not_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], "~~~~~").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_separated_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], ".~~.~#~.").unwrap();
    assert!(ol.line().verify());
}

#[test]
fn verify_separated_not_enough_space() {
    let ol = OwnedLine::create(vec![2, 3], ".~~.#~.").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_unsatisfialble_filled() {
    let ol = OwnedLine::create(vec![2, 3], "~~#~~~").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_unsatisfialble_filled_with_frame() {
    let ol = OwnedLine::create(vec![2, 3], ".~~#~~~.").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_split_with_badly_filled_left() {
    let ol = OwnedLine::create(vec![2, 3], "#~~#.~~~").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_too_many_filled() {
    let ol = OwnedLine::create(vec![2, 3], "#~~.~#~.#").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn verify_split_with_fine_left() {
    let ol = OwnedLine::create(vec![2, 3], "#~~#.~~").unwrap();
    assert!(!ol.line().verify());
}

#[test]
fn solve_simple_overlap_and_unreachable() {
    let ol = OwnedLine::create(vec![4], "~~~~~#~~").unwrap();
    let cache = Arc::new(RwLock::new(HashMap::new()));
    let result = ol.line().solve(cache).clone();
    let changes: HashSet<&Assumption> = result.iter().flat_map(|x| x.iter()).collect();
    assert_eq!(
        changes,
        HashSet::from([
            &Assumption { coords: (0, 0), val: Empty },
            &Assumption { coords: (0, 1), val: Empty },
            &Assumption { coords: (0, 4), val: Filled },
        ])
    );
}

#[test]
fn solve_fill_with_ambiguity() {
    let ol = OwnedLine::create(vec![1, 2], "~~~#.~~").unwrap();
    let cache = Arc::new(RwLock::new(HashMap::new()));
    let result = ol.line().solve(cache).clone();
    let changes: HashSet<&Assumption> = result.iter().flat_map(|x| x.iter()).collect();
    assert_eq!(changes, HashSet::from([&Assumption { coords: (0, 1), val: Empty },]));
}

#[test]
fn solve_empties_with_definite_chunks() {
    let ol = OwnedLine::create(vec![2, 1], "~~~.~#~.#").unwrap();
    let cache = Arc::new(RwLock::new(HashMap::new()));
    let result = ol.line().solve(cache).clone();
    let changes: HashSet<&Assumption> = result.iter().flat_map(|x| x.iter()).collect();
    assert_eq!(
        changes,
        HashSet::from([
            &Assumption { coords: (0, 0), val: Empty },
            &Assumption { coords: (0, 1), val: Empty },
            &Assumption { coords: (0, 2), val: Empty },
        ])
    );
}