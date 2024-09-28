use clap::Parser;
use nonogram::{Algorithm, SolutionResult, Solver};
use std::io;
use std::time::Instant;
use SolutionResult::*;

mod nonogram;

#[derive(Parser, Debug)]
struct Cli {
    fname: Option<String>,
    #[arg(value_enum, short, long, default_value_t = Algorithm::TwoSat)]
    algorithm: Algorithm,
    #[arg(short, long, default_value_t = 3, help("Max recusrion depth, 0 for no limit"))]
    max_depth: usize,
    #[arg(short, long)]
    find_all: bool,
}

fn main() {
    let cli = Cli::parse();
    let max_depth = if cli.max_depth > 0 { Some(cli.max_depth) } else { None };
    let solver = match cli.fname {
        Some(fname) => Solver::from_reader(
            std::fs::File::open(fname).unwrap(),
            max_depth,
            cli.find_all,
            cli.algorithm,
        )
        .unwrap(),
        None => Solver::from_reader(io::stdin(), max_depth, cli.find_all, cli.algorithm).expect("Malformed input"),
    };
    let start = Instant::now();
    match solver.solve() {
        Solved(fields) => {
            for fld in fields {
                println!("{}\n", fld.to_string());
            }
        }
        Unsolved(changes) => {
            let mut fld = solver.create_field();
            changes.iter().for_each(|ass| ass.apply(&mut fld));
            println!("Cannot solve; info so far: \n{}", fld.to_string())
        }
        Controversial => println!("Controversial"),
    }
    println!("Elapsed: {:?}", start.elapsed());
}
