use std::fs;
use std::env;

mod lexing;
mod parser;
mod errors;

#[cfg(test)]
mod test;

use lexing::*;
use errors::ParseResult;

use crate::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_filename = args.get(1).expect("No filename provided!");

    let file = fs::read(input_filename).expect("Unable to open file!");

    let mut parser = Parser::new(file);
    
    println!("{:?}", parser.parse().unwrap());
}
