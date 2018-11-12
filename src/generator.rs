use lexer::Operator;
use parser::{Expression, Factor, Function, Program, Statement, Term};
use std::fmt;

#[derive(Debug)]
pub enum Instruction {
    Directive(Directive, String),
    Label(String),
    NoArgInst(NoArgInst),
    SingleArgInst(SingleArgInst, Param),
    DoubleArgInst(DoubleArgInst, Param, Param),
}

trait ToAsm {
    fn to_asm(&self) -> String;
}

macro_rules! asm_from_name {
    ($enumname: ident { $($enumval: ident),*}) => {
        #[derive(Debug)]
        pub enum $enumname {
            $(
                $enumval,
            )*
        }
        impl ToAsm for $enumname {
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

asm_from_name!(Register {
    RAX,
    RBX,
    RCX,
    RDX,
    AL
});

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

pub fn generate(prog: &Program) -> Vec<Instruction> {
    generate_prog(&prog)
}

fn generate_prog(prog: &Program) -> Vec<Instruction> {
    generate_fn(&prog.function)
}

fn generate_fn(func: &Function) -> Vec<Instruction> {
    combine(
        vec![
            Instruction::Directive(Directive::Globl, func.name.clone()),
            Instruction::Label(func.name.clone()),
        ],
        generate_stmt(&func.statement),
    )
}

fn generate_stmt(stmt: &Statement) -> Vec<Instruction> {
    match stmt {
        Statement::Return(exp) => append(generate_exp(exp), Instruction::NoArgInst(NoArgInst::Ret)),
    }
}

fn generate_exp(exp: &Expression) -> Vec<Instruction> {
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
                Operator::Plus => vec![Instruction::DoubleArgInst(
                    DoubleArgInst::Add,
                    Param::Register(Register::RCX),
                    Param::Register(Register::RAX),
                )],
                Operator::Negate => vec![
                    Instruction::DoubleArgInst(
                        DoubleArgInst::Sub,
                        Param::Register(Register::RAX),
                        Param::Register(Register::RCX),
                    ),
                    Instruction::DoubleArgInst(
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
            l.push(Instruction::SingleArgInst(
                SingleArgInst::Push,
                Param::Register(Register::RAX),
            ));
            l.append(&mut r);
            l.push(Instruction::SingleArgInst(
                SingleArgInst::Pop,
                Param::Register(Register::RCX),
            ));
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
                Operator::Star => vec![Instruction::DoubleArgInst(
                    DoubleArgInst::Imul,
                    Param::Register(Register::RCX),
                    Param::Register(Register::RAX),
                )],
                Operator::Divide => vec![
                    Instruction::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Register(Register::RAX),
                        Param::Register(Register::RBX),
                    ),
                    Instruction::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Register(Register::RCX),
                        Param::Register(Register::RAX),
                    ),
                    Instruction::DoubleArgInst(
                        DoubleArgInst::Mov,
                        Param::Const(0),
                        Param::Register(Register::RDX),
                    ),
                    Instruction::SingleArgInst(SingleArgInst::Idiv, Param::Register(Register::RBX)),
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
            l.push(Instruction::SingleArgInst(
                SingleArgInst::Push,
                Param::Register(Register::RAX),
            ));
            l.append(&mut r);
            l.push(Instruction::SingleArgInst(
                SingleArgInst::Pop,
                Param::Register(Register::RCX),
            ));
            l.append(&mut op);
            l
        }
    }
}

fn generate_factor(factor: &Factor) -> Vec<Instruction> {
    match factor {
        Factor::Expression(exp) => generate_exp(exp),
        Factor::Const(int) => vec![Instruction::DoubleArgInst(
            DoubleArgInst::Mov,
            Param::Const(*int),
            Param::Register(Register::RAX),
        )],
        Factor::UnaryOperation { operator, factor } => {
            let factor_code = generate_factor(factor);
            match operator {
                Operator::Negate => append(
                    factor_code,
                    Instruction::SingleArgInst(SingleArgInst::Neg, Param::Register(Register::RAX)),
                ),
                Operator::Not => append(
                    factor_code,
                    Instruction::SingleArgInst(SingleArgInst::Not, Param::Register(Register::RAX)),
                ),
                Operator::Bang => combine(
                    factor_code,
                    vec![
                        Instruction::DoubleArgInst(
                            DoubleArgInst::Cmp,
                            Param::Const(0),
                            Param::Register(Register::RAX),
                        ),
                        Instruction::DoubleArgInst(
                            DoubleArgInst::Mov,
                            Param::Const(0),
                            Param::Register(Register::RAX),
                        ),
                        Instruction::SingleArgInst(
                            SingleArgInst::Sete,
                            Param::Register(Register::AL),
                        ),
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

/**
 * AT&AT Syntax
 */
pub fn write<T: std::io::Write>(instructions: &Vec<Instruction>, file: &mut T) {
    for inst in instructions {
        let _res = match inst {
            Instruction::Directive(dir, other) => writeln!(file, ".{} {}", dir.to_asm(), other),
            Instruction::Label(label) => writeln!(file, "{}:", label),
            Instruction::NoArgInst(inst) => writeln!(file, "\t{}", inst.to_asm()),
            Instruction::SingleArgInst(inst, arg) => {
                writeln!(file, "\t{}\t{}", inst.to_asm(), arg.to_string())
            },
            Instruction::DoubleArgInst(inst, src, dst) => writeln!(
                file,
                "\t{}\t{}, {}",
                inst.to_asm(),
                src.to_string(),
                dst.to_string()
            )
        };
    }
}

pub fn print_asm(assembly: &Vec<Instruction>) {
    for asm in assembly {
        println!("{:?}", asm);
    }
    println!();
}
