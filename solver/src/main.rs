use std::io;

mod nonogram;

use nonogram::Nonogram;

fn main() {
    let nono = Nonogram::from_reader(io::stdin()).expect("Malformed input");
    println!("{nono:?}");
}
