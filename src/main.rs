extern crate regex;

use std::collections::VecDeque;
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
    print_tokens(&tokens);
    parse(tokens);
}

#[derive(Debug)]
enum Keyword {
    Int,
    Return,
}

#[derive(Debug)]
enum Symbol {
    LeftBrace,
    RightBrace,
    LeftParenthesis,
    RightParenthesis,
    Semicolon,
}

#[derive(Debug)]
enum Operator {
    Negate,
    Not,
    Bang,
    Plus,
    Star,
    Divide,
}

#[derive(Debug)]
enum Integer {
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug)]
enum TokenType {
    Symbol {
        regex: &'static str,
        stype: Symbol,
    },
    Operator {
        regex: &'static str,
        otype: Operator,
    },
    Keyword {
        regex: &'static str,
        ktype: Keyword,
    },
    Integer {
        regex: &'static str,
        itype: Integer,
    },
    Identifier {
        regex: &'static str,
    },
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
            TokenType::Symbol { regex, .. }
            | TokenType::Operator { regex, .. }
            | TokenType::Keyword { regex, .. }
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

struct Token {
    ttype: &'static TokenType,
    value: Option<String>,
}

const RAW_PATTERNS: [TokenType; 16] = [
    TokenType::Symbol {
        regex: r"\{",
        stype: Symbol::LeftBrace,
    },
    TokenType::Symbol {
        regex: r"\}",
        stype: Symbol::RightBrace,
    },
    TokenType::Symbol {
        regex: r"\(",
        stype: Symbol::LeftParenthesis,
    },
    TokenType::Symbol {
        regex: r"\)",
        stype: Symbol::RightParenthesis,
    },
    TokenType::Symbol {
        regex: r";",
        stype: Symbol::Semicolon,
    },
    TokenType::Operator {
        regex: r"-",
        otype: Operator::Negate,
    },
    TokenType::Operator {
        regex: r"~",
        otype: Operator::Not,
    },
    TokenType::Operator {
        regex: r"!",
        otype: Operator::Bang,
    },
    TokenType::Operator {
        regex: r"\+",
        otype: Operator::Plus,
    },
    TokenType::Operator {
        regex: r"\*",
        otype: Operator::Star,
    },
    TokenType::Operator {
        regex: r"/",
        otype: Operator::Divide,
    },
    TokenType::Keyword {
        regex: r"int",
        ktype: Keyword::Int,
    },
    TokenType::Keyword {
        regex: r"return",
        ktype: Keyword::Return,
    },
    TokenType::Integer {
        regex: r"0x[0-9a-fA-F]+",
        itype: Integer::Hexadecimal,
    },
    // TokenType::Integer { regex: r"0[0-7]+", base: 8), // Might not be correct
    TokenType::Integer {
        regex: r"[0-9]+",
        itype: Integer::Decimal,
    },
    TokenType::Identifier {
        regex: r"[a-zA-Z]+",
    },
];

fn fail(message: &'static str) {
    panic!(message);
}

fn lex(code: &String) -> VecDeque<Token> {
    let mut source = code.as_str();

    let patterns: Vec<TokenDef> = RAW_PATTERNS.iter().map(TokenDef::create).collect();

    let mut tokens = std::collections::VecDeque::new();

    while !source.is_empty() {
        let mut found = false;
        for pattern in &patterns {
            let m = pattern.regex.find(&source);
            if m != None {
                let tok = m.unwrap();
                let val = match pattern.ttype {
                    TokenType::Integer { .. } | TokenType::Identifier { .. } => {
                        Some(String::from(tok.as_str().trim()))
                    }
                    _ => None,
                };

                tokens.push_back(Token {
                    value: val,
                    ttype: pattern.ttype,
                });
                source = &source[tok.end()..];
                found = true;
                break;
            }
        }

        if !found {
            fail("Unenpected token while lexing");
        }
    }

    tokens
}

struct Program {
    function: Function,
}

enum Statement {
    Return(Expression),
}

struct Function {
    name: String,
    statement: Statement,
}

struct UnaryOperation {
    operator: char,
    exp: Expression,
}

enum Expression {
    UnOp(Box<UnaryOperation>),
    Const(u32),
}

macro_rules! simple_match {
    ($tokens:expr, $type:pat) => {
        match $tokens.pop_front() {
            Some(Token { ttype: $type, .. }) => (),
            Some(Token { ttype, .. }) => panic!(format!("Unexpected token '{:?}'.", ttype)),
            _ => panic!("Unexpected Token while parsing"),
        }
    };
}
fn print_tokens(tokens: &VecDeque<Token>) {
    for tok in tokens {
        println!("{:?}", tok.ttype);
    }
}


fn parse(mut tokens: VecDeque<Token>) {
    parse_fn(&mut tokens);
}

fn parse_fn(mut tokens: &mut VecDeque<Token>) {
    let _return_type = match tokens.pop_front() {
        Some(Token {
            ttype:
                TokenType::Keyword {
                    ktype: Keyword::Int,
                    ..
                },
            ..
        }) => Keyword::Int,
        _ => panic!("Unexpected Token"),
    };

    let name = match tokens.pop_front() {
        Some(Token {
            ttype: TokenType::Identifier { .. },
            value,
        }) => value,
        _ => panic!("Unexpected Token"),
    };

    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::LeftParenthesis, .. });
    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::RightParenthesis, .. });
    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::LeftBrace, .. });

    let stmt = parse_statement(&mut tokens);

    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::RightBrace, .. });
}

fn parse_statement(mut tokens: &mut VecDeque<Token>) -> Statement {
    match tokens.pop_front() {
        Some(Token{ ttype:TokenType::Keyword {ktype: Keyword::Return, ..}, .. }) => {
            let exp = parse_exp(&mut tokens);
            simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::Semicolon, .. });
            Statement::Return(exp)
        },
        _ => panic!("Unexpected token in stmt")
    }
}

fn parse_exp(mut tokens: &mut VecDeque<Token>) -> Expression {
    // let tok = tokens.pop_front();
    match tokens.pop_front() {
        Some(Token { ttype: TokenType::Integer {itype, ..}, value: Some(ref num) }) => {
            let int = match itype {
                Integer::Decimal => num.parse::<u32>().unwrap(),
                Integer::Hexadecimal => {
                    u32::from_str_radix(num.trim_left_matches("0x"), 16).unwrap()
                }
            };
            let int : u32 = 12;
            return Expression::Const(int);
        },
        _ => panic!("Unexpected token in exp")
    }
}
