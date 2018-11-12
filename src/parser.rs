use lexer::{Integer, Keyword, Operator, Symbol, Token, TokenType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub statement: Statement,
}

#[derive(Debug)]
pub enum Expression {
    Term(Term),
    BinaryOperation {
        left: Box<Expression>,
        operator: Operator,
        right: Term,
    },
}

#[derive(Debug)]
pub enum Term {
    Factor(Factor),
    BinaryOperation {
        left: Box<Term>,
        operator: Operator,
        right: Factor,
    },
}

#[derive(Debug)]
pub enum Factor {
    Expression(Box<Expression>),
    UnaryOperation {
        operator: Operator,
        factor: Box<Factor>,
    },
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
    let mut exp = Expression::Term(parse_term(&mut tokens));
    loop {
        match tokens.get(0) {
            Some(Token {
                ttype: TokenType::Operator { otype, .. },
                ..
            }) => match otype {
                Operator::Plus | Operator::Negate => {
                    tokens.pop_front();
                    let next_term = parse_term(&mut tokens);
                    exp = Expression::BinaryOperation {
                        left: Box::new(exp),
                        operator: *otype,
                        right: next_term,
                    };
                }
                _ => {
                    break;
                }
            },
            _ => {
                break;
            }
        }
    }
    exp
}

fn parse_term(mut tokens: &mut VecDeque<Token>) -> Term {
    let mut term = Term::Factor(parse_factor(&mut tokens));
    loop {
        match tokens.get(0) {
            Some(Token {
                ttype: TokenType::Operator { otype, .. },
                ..
            }) => match otype {
                Operator::Star | Operator::Divide => {
                    tokens.pop_front();
                    let next_factor = parse_factor(&mut tokens);
                    term = Term::BinaryOperation {
                        left: Box::new(term),
                        operator: *otype,
                        right: next_factor,
                    };
                }
                _ => {
                    break;
                }
            },
            _ => {
                break;
            }
        }
    }
    term
}

fn parse_factor(mut tokens: &mut VecDeque<Token>) -> Factor {
    match tokens.pop_front() {
        Some(Token {
            ttype:
                TokenType::Symbol {
                    stype: Symbol::LeftParenthesis,
                    ..
                },
            ..
        }) => {
            let exp = parse_exp(&mut tokens);
            if let Some(Token {
                ttype:
                    TokenType::Symbol {
                        stype: Symbol::RightParenthesis,
                        ..
                    },
                ..
            }) = tokens.pop_front()
            {
                Factor::Expression(Box::new(exp))
            } else {
                panic!("Expected right parenthesis");
            }
        }
        Some(Token {
            ttype: TokenType::Operator { otype, .. },
            ..
        }) => match otype {
            Operator::Negate | Operator::Bang | Operator::Not => {
                let factor = parse_factor(&mut tokens);
                Factor::UnaryOperation {
                    operator: *otype,
                    factor: Box::new(factor),
                }
            }
            _ => panic!("Unexpected operator"),
        },
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
            Factor::Const(int)
        }
        _ => panic!("Unknown factor"),
    }
}

pub fn print_program(prog: &Program) {
    print_func(&prog.function);
}

fn print_func(func: &Function) {
    println!("FUNCTION {}:", func.name);
    print_stmt(&func.statement);
}

fn print_stmt(stmt: &Statement) {
    match stmt {
        Statement::Return(exp) => {
            let res = print_exp(exp);
            println!("\tRETURN {}", res);
        }
    };
}

fn print_exp(exp: &Expression) -> String {
    match exp {
        Expression::Term(term) => print_term(&term),
        Expression::BinaryOperation {
            left,
            operator,
            right,
        } => {
            let l = print_exp(left);
            let r = print_term(right);
            format!("{} {} {}", l, get_op(operator), r)
        }
    }
}

fn get_op(op: &Operator) -> &str {
    match op {
        Operator::Negate => "-",
        Operator::Not => "~",
        Operator::Bang => "!",
        Operator::Plus => "+",
        Operator::Star => "*",
        Operator::Divide => "/",
    }
}

fn print_term(term: &Term) -> String {
    match term {
        Term::Factor(factor) => print_factor(&factor),
        Term::BinaryOperation {
            left,
            operator,
            right,
        } => {
            let l = print_term(left);
            let r = print_factor(right);
            format!("{} {} {}", l, get_op(operator), r)
        }
    }
}

fn print_factor(factor: &Factor) -> String {
    match factor {
        Factor::Expression(exp) => format!("({})", print_exp(exp)),
        Factor::UnaryOperation { operator, factor } => {
            format!("{}{}", get_op(operator), print_factor(factor))
        }
        Factor::Const(int) => format!("{}", int),
    }
}
