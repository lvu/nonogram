use clap::Parser;
use nonogram::{Solution, Solver};
use std::io;
use std::time::Instant;
use Solution::*;

mod nonogram;

#[derive(Parser, Debug)]
struct Cli {
    fname: Option<String>,
    #[arg(short, long, default_value_t = 3, help("Max recusrion depth, 0 for no limit"))]
    max_depth: usize,
    #[arg(short, long)]
    find_all: bool,
}

fn main() {
    let cli = Cli::parse();
    let solver = match cli.fname {
        Some(fname) => Solver::from_reader(std::fs::File::open(fname).unwrap(), cli.max_depth, cli.find_all).unwrap(),
        None => Solver::from_reader(io::stdin(), cli.max_depth, cli.find_all).expect("Malformed input"),
    };
    let start = Instant::now();
    match solver.solve() {
        Solved(fields) => {
            for fld in fields {
                println!("{}\n", fld.to_string());
            }
        }
        Unsolved(fld) => {
            println!("Cannot solve; info so far: \n{}", fld.to_string())
        }
        Controversial => println!("Controversial"),
    }
    println!("Elapsed: {:?}", start.elapsed());
}
