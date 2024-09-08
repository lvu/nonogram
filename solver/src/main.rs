use std::io;
use std::time::Instant;

mod nonogram;

use nonogram::Nonogram;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut nono = if args.len() == 2 {
        Nonogram::from_reader(std::fs::File::open(args[1].clone()).unwrap()).unwrap()
    } else {
        Nonogram::from_reader(io::stdin()).expect("Malformed input")
    };
    let start = Instant::now();
    match nono.solve(3) {
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
