use lexer::Operator;
use parser::{Expression, Factor, Function, Program, Statement, Term};
use std::fmt;

#[derive(Debug)]
pub enum Instruction {
    global(String),
    mov { dst: Param, src: Param },
    ret,
    text,
    cmp { arg0: Param, arg1: Param },
    add { src: Param, dst: Param },
    sub { src: Param, dst: Param },
    imul { src: Param, dst: Param },
    sete { dst: Param },
    not { dst: Param },
    neg { dst: Param },
    push { src: Param },
    idiv { dst: Param },
    pop { dst: Param },
    label(String),
}

#[derive(Debug)]
pub enum Param {
    Const(u32),
    Str(String),
    Register(Register),
}

#[derive(Debug)]
pub enum Register {
    rax,
    rbx,
    rcx,
    rdx,
    al,
}

impl ToString for Param {
    fn to_string(&self) -> String {
        match self {
            Param::Const(int) => String::from(format!("${}", int)),
            Param::Str(string) => string.clone(),
            Param::Register(reg) => String::from(format!("%{}", reg)),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn generate(prog: &Program) -> Vec<Instruction> {
    generate_prog(&prog)
}

pub fn generate_prog(prog: &Program) -> Vec<Instruction> {
    generate_fn(&prog.function)
}

pub fn generate_fn(func: &Function) -> Vec<Instruction> {
    combine(
        vec![
            Instruction::global(func.name.clone()),
            Instruction::label(func.name.clone()),
        ],
        generate_stmt(&func.statement),
    )
}

pub fn generate_stmt(stmt: &Statement) -> Vec<Instruction> {
    match stmt {
        Statement::Return(exp) => append(generate_exp(exp), Instruction::ret),
    }
}

pub fn generate_exp(exp: &Expression) -> Vec<Instruction> {
    match exp {
        Expression::Term(term) => generate_term(term),
        Expression::BinaryOperation {
            left,
            operator,
            right,
        } => {
            let mut l = generate_exp(left);
            let mut r = generate_term(right);
            let mut op = match operator {
                Operator::Plus => vec![Instruction::add {
                    src: Param::Register(Register::rcx),
                    dst: Param::Register(Register::rax),
                }],
                Operator::Negate => vec![
                    Instruction::sub {
                        src: Param::Register(Register::rax),
                        dst: Param::Register(Register::rcx),
                    },
                    Instruction::mov {
                        src: Param::Register(Register::rcx),
                        dst: Param::Register(Register::rax),
                    },
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
            l.push(Instruction::push {
                src: Param::Register(Register::rax),
            });
            l.append(&mut r);
            l.push(Instruction::pop {
                dst: Param::Register(Register::rcx),
            });
            l.append(&mut op);
            l
        }
    }
}

fn generate_term(term: &Term) -> Vec<Instruction> {
    match term {
        Term::Factor(factor) => generate_factor(factor),
        Term::BinaryOperation {
            left,
            operator,
            right,
        } => {
            let mut l = generate_term(left);
            let mut r = generate_factor(right);
            let mut op = match operator {
                Operator::Star => vec![Instruction::imul {
                    src: Param::Register(Register::rcx),
                    dst: Param::Register(Register::rax),
                }],
                Operator::Divide => vec![
                    Instruction::mov {
                        src: Param::Register(Register::rax),
                        dst: Param::Register(Register::rbx),
                    },
                    Instruction::mov {
                        src: Param::Register(Register::rcx),
                        dst: Param::Register(Register::rax),
                    },
                    Instruction::mov {
                        src: Param::Const(0),
                        dst: Param::Register(Register::rdx),
                    },
                    Instruction::idiv {
                        dst: Param::Register(Register::rbx),
                    },
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
            l.push(Instruction::push {
                src: Param::Register(Register::rax),
            });
            l.append(&mut r);
            l.push(Instruction::pop {
                dst: Param::Register(Register::rcx),
            });
            l.append(&mut op);
            l
        }
    }
}

fn generate_factor(factor: &Factor) -> Vec<Instruction> {
    match factor {
        Factor::Expression(exp) => generate_exp(exp),
        Factor::Const(int) => vec![Instruction::mov {
            src: Param::Const(*int),
            dst: Param::Register(Register::rax),
        }],
        Factor::UnaryOperation { operator, factor } => match operator {
            Operator::Negate => append(
                generate_factor(factor),
                Instruction::neg {
                    dst: Param::Register(Register::rax),
                },
            ),
            Operator::Not => append(
                generate_factor(factor),
                Instruction::not {
                    dst: Param::Register(Register::rax),
                },
            ),
            Operator::Bang => combine(
                generate_factor(factor),
                vec![
                    Instruction::cmp {
                        arg0: Param::Const(0),
                        arg1: Param::Register(Register::rax),
                    },
                    Instruction::mov {
                        src: Param::Const(0),
                        dst: Param::Register(Register::rax),
                    },
                    Instruction::sete {
                        dst: Param::Register(Register::al),
                    },
                ],
            ),
            _ => panic!(format!("Unexpected operater '{:?}'", operator)),
        },
    }
}

fn combine<T>(mut left: Vec<T>, mut right: Vec<T>) -> Vec<T> {
    left.append(&mut right);
    left
}

fn prepend<T>(left: T, mut right: Vec<T>) -> Vec<T> {
    let mut pre = vec![left];
    pre.append(&mut right);
    pre
}

fn append<T>(mut left: Vec<T>, right: T) -> Vec<T> {
    left.push(right);
    left
}

/**
 * Formats a string to write to the given file,
 * calling to_string() on the parameters
 */
macro_rules! write_with_param {
    ($file:expr, $format:expr, $($arg:expr),*) => {
            writeln!($file, $format, $($arg.to_string(),)*)
    };
}

/**
 * AT&AT Syntax
 */
pub fn write<T: std::io::Write>(instructions: &Vec<Instruction>, file: &mut T) {
    for inst in instructions {
        let _res = match inst {
            Instruction::text => write_with_param!(file, ".text",),
            Instruction::global(name) => write_with_param!(file, ".globl {}", name),
            Instruction::label(name) => write_with_param!(file, "{}:", name),
            Instruction::ret => write_with_param!(file, "\tret",),
            Instruction::not { dst } => write_with_param!(file, "\tnot\t{}", dst),
            Instruction::neg { dst } => write_with_param!(file, "\tneg\t{}", dst),
            Instruction::sete { dst } => write_with_param!(file, "\tsete\t{}", dst),
            Instruction::pop { dst } => write_with_param!(file, "\tpop\t{}",  dst),
            Instruction::idiv { dst } => write_with_param!(file, "\tidiv\t{}",  dst),
            Instruction::push { src } => write_with_param!(file, "\tpush\t{}",  src),
            Instruction::mov { src, dst } => write_with_param!(file, "\tmov\t{}, {}", src, dst),
            Instruction::add { src, dst } => write_with_param!(file, "\tadd\t{}, {}", src, dst),
            Instruction::sub { src, dst } => write_with_param!(file, "\tsub\t{}, {}", src, dst),
            Instruction::imul { src, dst } => write_with_param!(file, "\timul\t{}, {}", src, dst),
            Instruction::cmp { arg0, arg1 } => write_with_param!(file, "\tcmp\t{}, {}", arg0, arg1),
            // _ => panic!("Not yet implemented"),
        };
    }
}

pub fn print_asm(assembly: &Vec<Instruction>) {
    for asm in assembly {
        println!("{:?}", asm);
    }
    println!();
}
