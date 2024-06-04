use std::{fs, path::PathBuf};

use clap::Parser;
use object::{ObjectPool, Value};
use token::Tokens;

mod eval;
mod object;
mod token;

/// Interpreter test program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    script: PathBuf,
}

fn main() {
    let args = Args::parse();
    let script = fs::read_to_string(args.script).expect("Unable to read file");
    let tokens = match Tokens::from_source(&script) {
        Ok(tokens) => tokens,
        Err(msg) => panic!("{msg}"),
    };
    println!("{tokens}");
}
