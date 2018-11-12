use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

mod generator;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let mut f = File::open(path).expect("file not found");
    let mut source = String::new();

    f.read_to_string(&mut source).expect("cannot read file");

    let debug = false;

    let tokens = lexer::lex(&source);
    if debug {
        lexer::print_tokens(&tokens);
    }

    let prog = parser::parse(tokens);
    if debug {
        parser::print_program(&prog);
    }

    let assembly = generator::generate(&prog);
    if debug {
        generator::print_asm(&assembly);
    }
    let filepath = PathBuf::from(path);
    if let Some(basename) = filepath.file_stem() {
        let basename = basename.to_str().unwrap();
        let mut asm_path = PathBuf::from(basename);
        asm_path.set_extension("s");

        if let Some(parent) = filepath.parent() {
            let mut pt = PathBuf::from(parent);
            pt.push(asm_path);
            asm_path = pt;
        }

        if let Ok(mut asm_file) = File::create(&asm_path) {
            generator::write(&assembly, &mut asm_file);
            let exe_path = match asm_path.parent() {
                Some(path) => {
                    let mut parent = PathBuf::from(path);
                    parent.push(basename);
                    parent
                }
                _ => PathBuf::from(&asm_path),
            };
            compile(asm_path, exe_path);
        }
    }
}

fn compile(asm_path: PathBuf, exe_path: PathBuf) {
    let output = if cfg!(target_os = "windows") {
        panic!("Cannot compile the assembly on windos")
    } else {
        Command::new("gcc")
            .arg(asm_path.to_str().unwrap())
            .arg("-o")
            .arg(exe_path.to_str().unwrap())
            .output()
            .expect("failed to compile")
    };
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Error:");
        println!("{}", stderr);
        panic!();
    }
}
