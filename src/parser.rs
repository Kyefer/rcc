use lexer::{Integer, Keyword, Operator, Symbol, Token, TokenType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Program {
    pub function: Function
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression)
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub statement: Statement
}

#[derive(Debug)]
pub enum Expression {
    UnaryOperation {
        operator: Operator,
        exp: Box<Expression>,
    },
    Const(u32)
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

pub fn print_program(prog: &Program) {
    println!("{:?}\n", prog);
}

pub fn parse(mut tokens: VecDeque<Token>) -> Program {
    Program {
        function: parse_fn(&mut tokens),
    }
}

fn parse_fn(mut tokens: &mut VecDeque<Token>) -> Function {
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
            value: Some(name),
        }) => name,
        _ => panic!("Unexpected Token"),
    };

    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::LeftParenthesis, .. });
    // Parse arguments
    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::RightParenthesis, .. });
    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::LeftBrace, .. });

    let stmt = parse_statement(&mut tokens);

    simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::RightBrace, .. });

    Function {
        name: name,
        statement: stmt,
    }
}

fn parse_statement(mut tokens: &mut VecDeque<Token>) -> Statement {
    match tokens.pop_front() {
        Some(Token {
            ttype:
                TokenType::Keyword {
                    ktype: Keyword::Return,
                    ..
                },
            ..
        }) => {
            let exp = parse_exp(&mut tokens);
            simple_match!(&mut tokens, TokenType::Symbol { stype: Symbol::Semicolon, .. });
            Statement::Return(exp)
        }
        _ => panic!("Unexpected token in stmt"),
    }
}

fn parse_exp(mut tokens: &mut VecDeque<Token>) -> Expression {
    match tokens.pop_front() {
        Some(Token {
            ttype: TokenType::Integer { itype, .. },
            value: Some(ref num),
        }) => {
            let int = match itype {
                Integer::Decimal => num.parse::<u32>().unwrap(),
                Integer::Hexadecimal => {
                    u32::from_str_radix(num.trim_left_matches("0x"), 16).unwrap()
                }
            };
            Expression::Const(int)
        }
        Some(Token {
            ttype: TokenType::Operator { otype, .. },
            ..
        }) => Expression::UnaryOperation {
            operator: *otype,
            exp: Box::new(parse_exp(&mut tokens)),
        },
        _ => panic!("Unexpected token in exp"),
    }
}
