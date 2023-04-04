use std::fs;
use std::env;
use std::process::Command;

#[cfg(test)]
mod test;
mod lexer;
mod parser;
mod compiler;

fn compile_file(input_filename: &str, output_path: &str) {
    use std::io::prelude::*;

    // let input_path = "mud_tests/".to_owned() + input_filename;

    let in_file = fs::read(&input_filename).expect(&format!("Unable to open file {}!", input_filename));
    // let mut lexer = Lexer::new(file);
    let mut comp = compiler::Compiler::new();

    let program = comp.compile_full(in_file).expect(&format!("Error compiling {input_filename}!"));

    let mut output_filename: String = input_filename.split(".").take(1).collect();
    output_filename += ".c";

    let outpath = output_path.to_string() + &output_filename;

    let mut out_file = fs::File::create(&outpath)
        .expect(&format!("Unable to create file {}", &outpath));
    out_file
        .write_all(&program)
        .expect("Unable to write to file");

    // let target_filename = "mud_tests/truth/".to_string() + &output_filename;
    // let target_file = fs::read(target_filename);
}

fn transpile_file(input_filename: &str) {
    compile_file(input_filename, "");

    let output_filename: String = input_filename.split(".").take(1).collect();
    let output_filename_c = output_filename.clone() + ".c";

    let output = Command::new("gcc")
        .arg( &output_filename_c)
        .arg("-o")
        .arg(output_filename + &".exe")
        .output()
        .expect("Failed to run compiler");

    if !output.status.success(){
        dbg!(&output.status);
    }
    // println!("command output: {}", String::from_utf8_lossy(&output.stdout));
    println!("compiler error/warnings: {}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "The transpiled code failed to compile"
    );

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_filename = args.get(1).expect("No filename provided!");

    transpile_file(input_filename);

}
