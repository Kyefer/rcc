use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod assembler;
mod exporter;
mod generator;
mod lexer;
mod parser;

static DEBUG_FLAG: &str = "RCC_DEBUG";

fn main() {
    let path = env::args().nth(1).expect("expected a file to compile");

    let debug = env::var(DEBUG_FLAG).is_ok();

    let mut f = File::open(&path).expect("file not found");

    let mut source = String::new();
    f.read_to_string(&mut source).expect("cannot read file");

    let tokens = lexer::lex(&source);
    if debug {
        lexer::debug(&tokens);
    }

    let prog = parser::parse(tokens);
    if debug {
        parser::debug(&prog);
    }

    // let syntax = generator::Syntax::ATT;
    let assembly = generator::generate(&prog, &generator::Syntax::ATT);
    if debug {
        generator::debug(&prog);
    }

    let mut asm_path = PathBuf::from(&path);
    asm_path.set_extension("s");

    exporter::to_file(&asm_path, &assembly);

    let mut exe_path = PathBuf::from(&path);
    exe_path.set_extension("");

    assembler::assemble(&asm_path, &exe_path);
}
