use std::env;
use std::fs::File;
use std::io::prelude::*;

mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut source = String::new();

    f.read_to_string(&mut source).expect("cannot read file");

    let tokens = lexer::lex(&source);
    // lexer::print_tokens(&tokens);
    let prog = parser::parse(tokens);
    // parser::print_program(prog);
}
