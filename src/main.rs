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
    Symbol { regex: &'static str },
    Operator { regex: &'static str },
    Keyword { regex: &'static str },
    Integer { regex: &'static str, base: u16 },
    Identifier { regex: &'static str },
}

impl TokenType {
    fn bound(&self) -> &'static str {
        match self {
            TokenType::Keyword { .. } => r"\b",
            _ => "",
        }
    }

    fn regex(&self) -> &'static str {
        match self {
            TokenType::Symbol { regex }
            | TokenType::Operator { regex }
            | TokenType::Keyword { regex }
            | TokenType::Identifier { regex }
            | TokenType::Integer { regex, .. } => regex,
        }
    }

    fn prepare(&self) -> String {
        format!("{}{}{}{}", r"^", self.regex(), self.bound(), r"\s*")
    }
}

struct TokenDef {
    ttype: &'static TokenType,
    regex: Regex,
}

impl TokenDef {
    fn create(ttype: &'static TokenType) -> TokenDef {
        TokenDef {
            ttype: ttype,
            regex: Regex::new(&ttype.prepare()).unwrap(),
        }
    }
}

const RAW_PATTERNS: [TokenType; 16] = [
    TokenType::Symbol { regex: r"\{" },
    TokenType::Symbol { regex: r"\}" },
    TokenType::Symbol { regex: r"\(" },
    TokenType::Symbol { regex: r"\)" },
    TokenType::Symbol { regex: r";" },
    TokenType::Operator { regex:r"-" },
    TokenType::Operator { regex: r"~" },
    TokenType::Operator { regex: r"!" },
    TokenType::Operator { regex: r"\+" },
    TokenType::Operator { regex: r"\*" },
    TokenType::Operator { regex: r"/" },
    TokenType::Keyword { regex: r"int\b" },
    TokenType::Keyword { regex: r"return\b" },
    TokenType::Integer { regex: r"0x[0-9a-fA-F]+", base: 16 },
    // TokenType::Integer { regex: r"0[0-7]+", base: 8), // Might not be correct
    TokenType::Integer { regex: r"[0-9]+", base: 10 },
    TokenType::Identifier { regex: r"[a-zA-Z]+" },
];

fn fail(message: &'static str) {
    panic!(message);
}

fn lex(code: &String) -> std::collections::VecDeque<(&str, &TokenType)> {
    let mut source = code.as_str();

    let patterns: Vec<TokenDef> = RAW_PATTERNS.iter().map(TokenDef::create).collect();

    let mut tokens = std::collections::VecDeque::new();

    while !source.is_empty() {
        for pattern in &patterns {
            let m = pattern.regex.find(&source);
            if m != None {
                let tok = m.unwrap();
                tokens.push_back((tok.as_str().trim(), pattern.ttype));
                source = &source[tok.end()..];
                println!("{}", tok.as_str().trim());
                break;
            }
        }
        fail("Unenpected token");
    }

    tokens
}

struct Program {
    function: Function,
}

enum Statement {
    Return(Return),
}

struct Function {
    name: String,
    statement: Statement,
}

struct Return {
    exp: Expression,
}

struct UnaryOperation {
    operator: char,
    exp: Expression,
}

enum Expression {
    UnOp(Box<UnaryOperation>),
    Const(i32),
}

fn parse(mut tokens: std::collections::VecDeque<(&str, &TokenType)>) {
    parse_fn(tokens);
}

fn parse_fn(mut tokens: std::collections::VecDeque<(&str, &TokenType)>) {

    // let tok = tokens.pop_front().unwrap();
    // match (tok.0, tok.1) {
    //     ("int", TokenType::Keyword) => (),
    //     _ => panic!("Expected the 'int' keyword")
    // }

    // let tok = tokens.pop_front().unwrap();
    // let name = match tok.1 {
    //     TokenType::Identifier => tok.0,
    //     _ => panic!("Expected an identifier")
    // };

    // Function {
    //     name: String::from(name),
    //     statement: String::from("")
    // }
}
