use crate::lexer::{Lexer, Lexeme};
use crate::*;

fn parse_file(input_filename: &str){
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut parser = Parser::new(file);

    println!("{:?}", parser.parse().unwrap());
}

fn lex_file(input_filename: &str){
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut lexer = Lexer::new(file);
    // println!("{:?}", parser.parse().unwrap());

    loop {
        let token = lexer.next();
        dbg!(&token);
        if let Ok(Lexeme::Eof) = token {
            break
        }
    }
}

fn compile_file(input_filename: &str){
    use std::fs::File;
    use std::io::prelude::*;

    let input_path = "mud_tests/".to_owned() + input_filename;

    let in_file = fs::read(&input_path).expect("Unable to open file!");
    // let mut lexer = Lexer::new(file);
    let program = compiler::compile(in_file).expect("Error Compiling!");

    let mut output_filename: String = input_filename.split(".").take(1).collect();
    output_filename += ".c";

    dbg!(&output_filename);
    let mut out_file = fs::File::create("mud_tests/target/".to_string() + &output_filename).expect("Unable to create file");
    out_file.write_all(&program).expect("Unable to write to file");

}


#[test]
fn add_mul(){
    let filename = "add_mul.mud";

    lex_file(filename);
    parse_file(filename);
    compile_file(filename);
}
