use crate::*;

fn parse_file(input_filename: &str){
    let input_filename = "mud_tests/".to_owned() + input_filename;

    let file = fs::read(input_filename).expect("Unable to open file!");
    let mut parser = Parser::new(file);

    println!("{:?}", parser.parse().unwrap());
}

#[test]
fn add_mul(){
    let filename = "add_mul.mud";
    parse_file(filename);
}
