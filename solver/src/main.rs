use std::io;
use std::time::Instant;

mod nonogram;

use nonogram::Nonogram;

fn main() {
    let mut nono = Nonogram::from_reader(io::stdin()).expect("Malformed input");
    let start = Instant::now();
    match nono.solve(2) {
        Ok(fields) => for (fld, all_assumptions) in fields {
            println!("{fld}");
            for assumptions in all_assumptions {
                println!("{assumptions:?}");
            }
        },
        Err(err) => println!("Error: {err}")
    }
    println!("Elapsed: {:?}", start.elapsed());
}
