use crate::lexer::{Lexeme, Lexer};
use crate::*;
use std::process::Command;

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

fn compile_file(input_filename: &str) {
    use std::io::prelude::*;

    let input_path = "mud_tests/".to_owned() + input_filename;

    let in_file = fs::read(&input_path).expect("Unable to open file!");
    // let mut lexer = Lexer::new(file);
    let mut comp = compiler::Compiler::new();

    let program = comp.compile_full(in_file).expect(&format!("Error compiling {input_filename}!"));

    let mut output_filename: String = input_filename.split(".").take(1).collect();
    output_filename += ".c";

    let mut out_file = fs::File::create("mud_tests/target/".to_string() + &output_filename)
        .expect("Unable to create file");
    out_file
        .write_all(&program)
        .expect("Unable to write to file");

    // let target_filename = "mud_tests/truth/".to_string() + &output_filename;
    // let target_file = fs::read(target_filename);
}

fn compile_run_file(input_filename: &str, expected_output: &str) {
    compile_file(input_filename);

    let mut output_filename: String = input_filename.split(".").take(1).collect();
    let output_filename_c = output_filename.clone() + ".c";

    let output = Command::new("gcc")
        .arg("mud_tests/target/".to_string() + &output_filename_c)
        .arg("-o")
        .arg("mud_tests/target/".to_string() + &output_filename + &".exe")
        .output()
        .expect("Failed to run compiler");

    if !output.status.success(){
        dbg!(&output.status);
    }
    // println!("command output: {}", String::from_utf8_lossy(&output.stdout));
    println!("compiler error: {}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "The transpiled code failed to compile"
    );

    let output = Command::new("./".to_string() +
                              &"mud_tests/target/" + &output_filename + &".exe")
        .output()
        .expect("Failed to run program");

    if !output.status.success(){
        dbg!(&output.status);
    }

    println!("run error: {}", String::from_utf8_lossy(&output.stderr));
    println!("run output: {}", String::from_utf8_lossy(&output.stdout));

    assert!(
        output.status.success(),
        "The transpiled code crashed"
    );
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

// #[test]
fn identifiers(){
    // parse_file("identifiers.mud");
    compile_file("identifiers.mud");
}

#[test]
fn assignment(){
    lex_file("assignment.mud");
    parse_file("assignment.mud");
    compile_file("assignment.mud");
}

#[test]
fn print(){
    lex_file("print.mud");
    compile_file("print.mud");
}

#[test]
fn scope(){
    lex_file("scope.mud");
    compile_file("scope.mud");
}

#[test]
fn run_if_else() {
    let filename = "if_else.mud";
    compile_run_file(filename, "");
}


// #[test]
fn run_add_mul() {
    let filename = "add_mul.mud";
    compile_run_file(filename, "15");
}
