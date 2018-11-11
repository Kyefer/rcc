use lexer::Operator;
use parser::{Expression, Function, Program, Statement};
use std::fmt;

#[derive(Debug)]
pub enum Instruction {
    global(String),
    movl { arg0: Param, arg1: Param },
    ret,
    cmpl { arg0: Param, arg1: Param },
    sete { arg0: Param },
    not { arg0: Param },
    neg { arg0: Param },
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
    eax,
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

pub enum InstructionSet {
    x86,
}

pub fn generate(prog: &Program, inst_set: InstructionSet) -> Vec<Instruction> {
    match inst_set {
        InstructionSet::x86 => generate_prog(&prog),
    }
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
        Expression::Const(int) => vec![Instruction::movl {
            arg0: Param::Const(*int),
            arg1: Param::Register(Register::eax),
        }],
        Expression::UnaryOperation { operator, exp } => match operator {
            Operator::Negate => append(
                generate_exp(exp),
                Instruction::neg {
                    arg0: Param::Register(Register::eax),
                },
            ),
            Operator::Not => append(
                generate_exp(exp),
                Instruction::not {
                    arg0: Param::Register(Register::eax),
                },
            ),
            Operator::Bang => combine(
                generate_exp(exp),
                vec![
                    Instruction::cmpl {
                        arg0: Param::Const(0),
                        arg1: Param::Register(Register::eax),
                    },
                    Instruction::movl {
                        arg0: Param::Const(0),
                        arg1: Param::Register(Register::eax),
                    },
                    Instruction::sete {
                        arg0: Param::Register(Register::al),
                    },
                ],
            ),
            _ => panic!("Not implemented yet"),
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

macro_rules! write_with_param {
    ($file:expr, $format:expr, $($arg:expr),*) => {
        {
            write!($file, $format, $($arg.to_string(),)*);
            writeln!($file)
        }
    };
}

pub fn write<T: std::io::Write>(instructions: &Vec<Instruction>, file: &mut T) {
    for inst in instructions {
        let res = match inst {
            Instruction::global(name) => write_with_param!(file, ".globl {}", name),
            Instruction::label(name) => write_with_param!(file, ".{}:", name),
            Instruction::ret => write_with_param!(file, "ret",),
            Instruction::not { arg0 } => write_with_param!(file, "not {}", arg0),
            Instruction::neg { arg0 } => write_with_param!(file, "neg {}", arg0),
            Instruction::sete { arg0 } => write_with_param!(file, "sete {}", arg0),
            Instruction::movl { arg0, arg1 } => write_with_param!(file, "movl {} {}", arg0, arg1),
            Instruction::cmpl { arg0, arg1 } => write_with_param!(file, "cmpl {} {}", arg0, arg1),
            _ => panic!("Not yet implemented"),
        };
    }
}

pub fn print_asm(assembly: &Vec<Instruction>) {
    for asm in assembly {
        println!("{:?}", asm);
    }
    println!();
}
