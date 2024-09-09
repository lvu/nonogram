use std::io;
use std::time::Instant;

mod nonogram;

use nonogram::{Nonogram, Solved, Unsolved, Controversial};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut nono = if args.len() == 2 {
        Nonogram::from_reader(std::fs::File::open(args[1].clone()).unwrap()).unwrap()
    } else {
        Nonogram::from_reader(io::stdin()).expect("Malformed input")
    };
    let start = Instant::now();
    match nono.solve(4, true) {
        Solved(fields) => for fld in fields {
            println!("{fld}\n");
        },
        Unsolved => println!("Cannot solve; info so far: \n{}", nono.field_as_string()),
        Controversial => println!("Controversial")
    }
    println!("Elapsed: {:?}", start.elapsed());
}
