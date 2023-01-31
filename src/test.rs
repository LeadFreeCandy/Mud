use crate::lexer::{Lexeme, Lexer};
use crate::*;

fn parse_file(input_filename: &str) {
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut parser = Parser::new(file);

    println!("{:?}", parser.parse().unwrap());
}

fn lex_file(input_filename: &str) {
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut lexer = Lexer::new(file);
    // println!("{:?}", parser.parse().unwrap());

    loop {
        let token = lexer.next();
        dbg!(&token);
        if let Ok(Lexeme::Eof) = token {
            break;
        }
    }
}

fn compile_file(input_filename: &str) {
    use std::io::prelude::*;

    let input_path = "mud_tests/".to_owned() + input_filename;

    let in_file = fs::read(&input_path).expect("Unable to open file!");
    // let mut lexer = Lexer::new(file);
    let program = compiler::compile(in_file).expect(&format!("Error compiling {input_filename}!"));

    let mut output_filename: String = input_filename.split(".").take(1).collect();
    output_filename += ".c";

    dbg!(&output_filename);
    let mut out_file = fs::File::create("mud_tests/target/".to_string() + &output_filename)
        .expect("Unable to create file");
    out_file
        .write_all(&program)
        .expect("Unable to write to file");

    // let target_filename = "mud_tests/truth/".to_string() + &output_filename;
    // let target_file = fs::read(target_filename);
}

#[test]
fn add_mul() {
    let filename = "add_mul.mud";
    compile_file(filename);
}

#[test]
fn sequence() {
    let filename = "sequence.mud";
    compile_file(filename);
}
