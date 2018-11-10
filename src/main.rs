extern crate regex;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut source = String::new();

    f.read_to_string(&mut source).expect("cannot read file");

    let tokens = lex(&source);
    parse(tokens);
}

enum TokenType {
    Keyword,
    Symbol,
    Operator,
    Integer,
    Identifier,
}




enum TokenType2<'a> {
    Symbol(&'a str),
    Operator(&'a str),
    Keyword(&'a str),
    Integer(&'a str, u16),
    Identifier(&'a str)
}



fn lex(code: &String) -> std::collections::VecDeque<(&str, &TokenType)> {
    let mut source = code.as_str();

    let raw_patterns = [
        (r"\{", &TokenType::Symbol),
        (r"\}", &TokenType::Symbol),
        (r"\(", &TokenType::Symbol),
        (r"\)", &TokenType::Symbol),
        (r";", &TokenType::Symbol),
        (r"-", &TokenType::Symbol),
        (r"~", &TokenType::Operator),
        (r"!", &TokenType::Operator),
        (r"\+", &TokenType::Operator),
        (r"\*", &TokenType::Operator),
        (r"/", &TokenType::Operator),
        (r"int", &TokenType::Keyword),
        (r"return", &TokenType::Keyword),
        (r"0x[0-9a-fA-F]+", &TokenType::Integer),
        (r"0[0-7]+", &TokenType::Integer),
        (r"[0-9]+", &TokenType::Integer),
        (r"[a-zA-Z]", &TokenType::Identifier),
    ];

    // let test = [
    //     TokenType2::Symbol(SimpleToken {regex: r"\{"})
    // ];



    let mut patterns = Vec::with_capacity(raw_patterns.len());

    for raw_pattern in raw_patterns.iter() {
        let whitespace = match &raw_pattern.1 {
            TokenType::Keyword => r"\s+",
            _ => r"\s*",
        };
        let regex = format!(r"^{}{}", raw_pattern.0, whitespace);
        patterns.push((Regex::new(&regex).unwrap(), raw_pattern.1));
    }

    let mut tokens = std::collections::VecDeque::new();

    while !source.is_empty() {
        for pattern in &patterns {
            let m = pattern.0.find(&source);
            if m != None {
                let tok = m.unwrap();
                tokens.push_back((tok.as_str().trim(), pattern.1));
                source = &source[tok.end()..];
                break;
            }
        }
    }

    tokens
}


struct Program {
    function: Function
}

enum Statement {
    Return(Return)
}

struct Function {
    name: String,
    statement: Statement
}

struct Return {
    exp: Expression
}

struct UnaryOperation {
    operator: char,
    exp: Expression
}

enum Expression {
    UnOp(Box<UnaryOperation>),
    Const(i32)
}



fn parse(mut tokens: std::collections::VecDeque<(&str, &TokenType)>) {
    parse_fn(tokens);
}

fn parse_fn(mut tokens: std::collections::VecDeque<(&str, &TokenType)>) {
    
    let tok = tokens.pop_front().unwrap();
    match (tok.0, tok.1) {
        ("int", TokenType::Keyword) => (),
        _ => panic!("Expected the 'int' keyword")
    }
    
    let tok = tokens.pop_front().unwrap();
    let name = match tok.1 {
        TokenType::Identifier => tok.0,
        _ => panic!("Expected an identifier")
    };

    // Function {
    //     name: String::from(name),
    //     statement: String::from("")
    // }
}
