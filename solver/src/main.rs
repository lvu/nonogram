use std::io;
use std::time::Instant;
use clap::Parser;

mod nonogram;

use nonogram::{Nonogram, Solved, Unsolved, Controversial};

#[derive(Parser, Debug)]
struct Cli {
    fname: Option<String>,
    #[arg(short, long, default_value_t = 3, help("Max recusrion depth, 0 for no limit"))] max_depth: usize,
    #[arg(short = 'a', long)] find_all: bool,
}

fn main() {
    let cli = Cli::parse();
    let args: Vec<String> = std::env::args().collect();
    let mut nono = match cli.fname {
        Some(fname) => Nonogram::from_reader(std::fs::File::open(fname).unwrap()).unwrap(),
        None => Nonogram::from_reader(io::stdin()).expect("Malformed input")
    };
    let start = Instant::now();
    match nono.solve(if cli.max_depth > 0 { Some(cli.max_depth) } else { None }, cli.find_all) {
        Solved(fields) => for fld in fields {
            println!("{fld}\n");
        },
        Unsolved => println!("Cannot solve; info so far: \n{}", nono.field_as_string()),
        Controversial => println!("Controversial")
    }
    println!("Elapsed: {:?}", start.elapsed());
}
