use clap::Parser;
use std::io;
use std::time::Instant;

mod nonogram;

use nonogram::{Controversial, PartiallySolved, Solved, Solver, Unsolved};

#[derive(Parser, Debug)]
struct Cli {
    fname: Option<String>,
    #[arg(short, long, default_value_t = 3, help("Max recusrion depth, 0 for no limit"))]
    max_depth: usize,
    #[arg(short = 'a', long)]
    find_all: bool,
}

fn main() {
    let cli = Cli::parse();
    let max_depth = if cli.max_depth > 0 { Some(cli.max_depth) } else { None };
    let solver = match cli.fname {
        Some(fname) => Solver::from_reader(std::fs::File::open(fname).unwrap(), max_depth, cli.find_all).unwrap(),
        None => Solver::from_reader(io::stdin(), max_depth, cli.find_all).expect("Malformed input"),
    };
    let start = Instant::now();
    match solver.solve_2sat() {
        Solved(fields) => {
            for fld in fields {
                println!("{}\n", fld.to_string());
            }
        }
        Unsolved => println!("Cannot solve at all"),
        PartiallySolved(fld) => println!("Cannot solve; info so far: \n{}", fld.to_string()),
        Controversial => println!("Controversial"),
    }
    println!("Elapsed: {:?}", start.elapsed());
}
