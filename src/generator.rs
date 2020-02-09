use lexer::Operator;
use parser::{Expression, Factor, Function, Program, Statement, Term};
use std::fmt;

#[derive(Debug)]
enum Line {
    Directive(Directive, String),
    Label(String),
    NoArgInst(NoArgInst),
    SingleArgInst(SingleArgInst, Param),
    DoubleArgInst(DoubleArgInst, Param, Param),
}

#[derive(Debug)]
pub enum Syntax {
    ATT,
}

impl Line {
    fn apply_syntax(&self, syntax: &Syntax) -> String {
        match syntax {
            Syntax::ATT => match self {
                Line::Directive(dir, other) => format!(".{} {}", dir.to_asm(), other),
                Line::Label(label) => format!("{}:", label),
                Line::NoArgInst(inst) => format!("\t{}", inst.to_asm()),
                Line::SingleArgInst(inst, arg) => format!("\t{}\t{}", inst.to_asm(), arg.to_string()),
                Line::DoubleArgInst(inst, src, dst) => {
                    format!("\t{}\t{}, {}", inst.to_asm(), src.to_string(), dst.to_string())
                }
            },
        }
    }
}

macro_rules! asm_from_name {
    ($enumname: ident { $($enumval: ident),*}) => {
        #[derive(Debug)]
        pub enum $enumname {
            $(
                $enumval,
            )*
        }
        impl $enumname {
            fn to_asm(&self) -> String {
                match self {
                    $(
                        $enumname::$enumval => String::from(stringify!($enumval)).to_lowercase(),
                    )*
                }
            }
        }
    };
}

asm_from_name!(Directive { Globl });
asm_from_name!(NoArgInst { Ret });
asm_from_name!(SingleArgInst {
    Sete,
    Not,
    Neg,
    Push,
    Idiv,
    Pop
});
asm_from_name!(DoubleArgInst {
    Mov,
    Cmp,
    Add,
    Sub,
    Imul
});

asm_from_name!(Register { RAX, RBX, RCX, RDX, AL });

#[derive(Debug)]
pub enum Param {
    Const(u32),
    // Str(String),
    Register(Register),
}

impl ToString for Param {
    fn to_string(&self) -> String {
        match self {
            Param::Const(int) => String::from(format!("${}", int)),
            // Param::Str(string) => string.clone(),
            Param::Register(reg) => String::from(format!("%{}", reg.to_asm())),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn generate(prog: &Program, syntax: &Syntax) -> Vec<String> {
    generate_prog(&prog)
        .iter()
        .map(|inst| inst.apply_syntax(&syntax))
        .collect()
}

pub fn debug(prog: &Program) {
    for asm in generate_prog(&prog) {
        println!("{:?}", asm);
    }
    println!();
}

fn generate_prog(prog: &Program) -> Vec<Line> {
    generate_fn(&prog.function)
}

fn generate_fn(func: &Function) -> Vec<Line> {
    combine(
        vec![
            Line::Directive(Directive::Globl, func.name.clone()),
            Line::Label(func.name.clone()),
        ],
        generate_stmt(&func.statement),
    )
}

fn generate_stmt(stmt: &Statement) -> Vec<Line> {
    match stmt {
        Statement::Return(exp) => append(generate_exp(exp), Line::NoArgInst(NoArgInst::Ret)),
    }
}

fn generate_exp(exp: &Expression) -> Vec<Line> {
    match exp {
        Expression::Term(term) => generate_term(term),
        Expression::BinaryOperation { left, operator, right } => {
            let mut l = generate_exp(left);
            let mut r = generate_term(right);
            let mut op = match operator {
                Operator::Plus => vec![Line::DoubleArgInst(
                    DoubleArgInst::Add,
                    Param::Register(Register::RCX),
                    Param::Register(Register::RAX),
                )],
                Operator::Negate => vec![
                    Line::DoubleArgInst(
                        DoubleArgInst::Sub,
                        Param::Register(Register::RAX),
                        Param::Register(Register::RCX),
                    ),
                    Line::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Register(Register::RCX),
                        Param::Register(Register::RAX),
                    ),
                ],
                _ => panic!(format!("Unexpected operater '{:?}'", operator)),
            };

            /*
               <code for left side>
               pushl %eax
               <code for right side>
               popl %ecx
               <code for operation>
            */
            l.push(Line::SingleArgInst(SingleArgInst::Push, Param::Register(Register::RAX)));
            l.append(&mut r);
            l.push(Line::SingleArgInst(SingleArgInst::Pop, Param::Register(Register::RCX)));
            l.append(&mut op);
            l
        }
    }
}

fn generate_term(term: &Term) -> Vec<Line> {
    match term {
        Term::Factor(factor) => generate_factor(factor),
        Term::BinaryOperation { left, operator, right } => {
            let mut l = generate_term(left);
            let mut r = generate_factor(right);
            let mut op = match operator {
                Operator::Star => vec![Line::DoubleArgInst(
                    DoubleArgInst::Imul,
                    Param::Register(Register::RCX),
                    Param::Register(Register::RAX),
                )],
                Operator::Divide => vec![
                    Line::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Register(Register::RAX),
                        Param::Register(Register::RBX),
                    ),
                    Line::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Register(Register::RCX),
                        Param::Register(Register::RAX),
                    ),
                    Line::DoubleArgInst(DoubleArgInst::Mov, Param::Const(0), Param::Register(Register::RDX)),
                    Line::SingleArgInst(SingleArgInst::Idiv, Param::Register(Register::RBX)),
                ],
                _ => panic!(format!("Unexpected operater '{:?}'", operator)),
            };

            /*
               <code for left side>
               pushl %eax
               <code for right side>
               popl %ecx
               <code for operation>
            */
            l.push(Line::SingleArgInst(SingleArgInst::Push, Param::Register(Register::RAX)));
            l.append(&mut r);
            l.push(Line::SingleArgInst(SingleArgInst::Pop, Param::Register(Register::RCX)));
            l.append(&mut op);
            l
        }
    }
}

fn generate_factor(factor: &Factor) -> Vec<Line> {
    match factor {
        Factor::Expression(exp) => generate_exp(exp),
        Factor::Const(int) => vec![Line::DoubleArgInst(
            DoubleArgInst::Mov,
            Param::Const(*int),
            Param::Register(Register::RAX),
        )],
        Factor::UnaryOperation { operator, factor } => {
            let factor_code = generate_factor(factor);
            match operator {
                Operator::Negate => append(
                    factor_code,
                    Line::SingleArgInst(SingleArgInst::Neg, Param::Register(Register::RAX)),
                ),
                Operator::Not => append(
                    factor_code,
                    Line::SingleArgInst(SingleArgInst::Not, Param::Register(Register::RAX)),
                ),
                Operator::Bang => combine(
                    factor_code,
                    vec![
                        Line::DoubleArgInst(DoubleArgInst::Cmp, Param::Const(0), Param::Register(Register::RAX)),
                        Line::DoubleArgInst(DoubleArgInst::Mov, Param::Const(0), Param::Register(Register::RAX)),
                        Line::SingleArgInst(SingleArgInst::Sete, Param::Register(Register::AL)),
                    ],
                ),
                _ => panic!(format!("Unexpected operater '{:?}'", operator)),
            }
        }
    }
}

fn combine<T>(mut left: Vec<T>, mut right: Vec<T>) -> Vec<T> {
    left.append(&mut right);
    left
}

// fn prepend<T>(left: T, mut right: Vec<T>) -> Vec<T> {
//     let mut pre = vec![left];
//     pre.append(&mut right);
//     pre
// }

fn append<T>(mut left: Vec<T>, right: T) -> Vec<T> {
    left.push(right);
    left
}
