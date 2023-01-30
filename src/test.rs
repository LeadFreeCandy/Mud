use crate::*;

fn parse_file(input_filename: &str){
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut parser = Parser::new(file);

    println!("{:?}", parser.parse().unwrap());
}

fn lex_files(input_filename: &str){
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

#[test]
fn add_mul(){
    let filename = "add_mul.mud";
    lex_files(filename);
    parse_file(filename);
}
