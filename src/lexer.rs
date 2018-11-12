extern crate regex;
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Keyword {
    Int,
    Return,
}

#[derive(Debug)]
pub enum Symbol {
    LeftBrace,
    RightBrace,
    LeftParenthesis,
    RightParenthesis,
    Semicolon,
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Negate,
    Not,
    Bang,
    Plus,
    Star,
    Divide,
}

#[derive(Debug)]
pub enum Integer {
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug)]
pub enum TokenType {
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
    regex: regex::Regex,
}

impl TokenDef {
    fn create(ttype: &'static TokenType) -> TokenDef {
        TokenDef {
            ttype: ttype,
            regex: regex::Regex::new(&ttype.prepare()).unwrap(),
        }
    }
}

pub struct Token {
    pub ttype: &'static TokenType,
    pub value: Option<String>,
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

pub fn lex(code: &String) -> VecDeque<Token> {
    let mut source = code.as_str();

    let patterns: Vec<TokenDef> = RAW_PATTERNS.iter().map(TokenDef::create).collect();

    let mut tokens = std::collections::VecDeque::new();
    source = source.trim();
    while !source.is_empty() {
        let mut found = false;
        for pattern in &patterns {
            if let Some(tok) =  pattern.regex.find(&source){
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
            println!("{}", source);
            panic!("Unenpected token while lexing");
        }
    }

    tokens
}

pub fn print_tokens(tokens: &VecDeque<Token>) {
    for tok in tokens {
        println!("{:?}", tok.ttype);
    }
}