use crate::lexer::{Lexeme, Lexer};
use crate::*;
use std::process::Command;

use crate::parser::Parser;

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
        println!("{:?}", token);
        if let Ok(Lexeme::Eof) = token {
            break;
        }
    }
}

fn test_compile(test_name: &str){
    let input_filepath = "mud_tests/".to_string() + test_name;
    compile_file(&input_filepath, "");
}

fn test_transpile(test_name: &str){
    let input_filepath = "mud_tests/".to_string() + test_name;
    transpile_file(&input_filepath);
}


fn test_run(test_name: &str, expected_out: Option<&str>){
    let input_filepath = "mud_tests/".to_string() + test_name;

    transpile_file(&input_filepath);
    let output_filename: String = test_name.split(".").take(1).collect();
    let output = Command::new("./".to_string() +
                              &"mud_tests/" + &output_filename + &".exe")
        .output()
        .expect("Failed to run program");

    if !output.status.success(){
        dbg!(&output.status);
    }

    println!("run error: {}", String::from_utf8_lossy(&output.stderr));
    println!("run output: {}", String::from_utf8_lossy(&output.stdout));

    if let Some(expected_out) = expected_out{
        assert!(expected_out == String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "The transpiled code crashed"
    );
}


#[test]
fn add_mul() {
    let filename = "add_mul.mud";
    test_compile(filename);
    test_transpile(filename);
    test_run(filename, None);
}

#[test]
fn sequence() {
    let filename = "sequence.mud";
    test_compile(filename);
}

// #[test]
fn identifiers(){
    // parse_file("identifiers.mud");
    test_compile("identifiers.mud");
}

#[test]
fn assignment(){
    lex_file("assignment.mud");
    parse_file("assignment.mud");
    test_compile("assignment.mud");
}

#[test]
fn print(){
    lex_file("print.mud");
    test_compile("print.mud");
    test_run("print.mud", Some("5"));
}

#[test]
fn scope(){
    lex_file("scope.mud");
    test_compile("scope.mud");
}

#[test]
fn run_if_else() {
    let filename = "if_else.mud";
    compile_run_file(filename, "");
}

#[test]
fn run_while() {
    let filename = "while.mud";
    compile_run_file(filename, "");
}



// #[test]
fn run_add_mul() {
    let filename = "add_mul.mud";
    transpile_file(filename);
}
