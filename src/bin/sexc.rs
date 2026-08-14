use binsex::lexer::Token;
use binsex::{Binary, Opcode};
use clap::Parser;
use colored::Colorize;
use logos::Logos;
use std::collections::HashMap;
use std::process::exit;
fn line_of(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset].matches('\n').count() + 1
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    Add,
    Sub,
    Dup,
    Pop,
    Mul,
    Mod,
    Div,
    Load,
    Store,
    Swap,
    Eq,
    Lt,
    Gte,
    Print(u8),
    Ret,
    Push(u8),
    Jmp(String),
    Jz(String),
}

impl Instr {
    fn size(&self) -> u16 {
        match self {
            Instr::Add
            | Instr::Sub
            | Instr::Dup
            | Instr::Swap
            | Instr::Mul
            | Instr::Pop
            | Instr::Ret
            | Instr::Mod
            | Instr::Load
            | Instr::Store
            | Instr::Div
            | Instr::Eq
            | Instr::Gte
            | Instr::Lt => 1,
            Instr::Push(_) | Instr::Print(_) => 2,
            Instr::Jmp(_) | Instr::Jz(_) => 3,
        }
    }
}
#[derive(Debug, Clone)]
pub enum Node {
    Label(String, usize),
    Instr(Instr, usize),
}

pub type Program = Vec<Node>;

enum Pending {
    None,
    Print,
    Push,
    PushMulti,
    PushStr,
    Jmp,
    Jz,
    Point,
}

pub fn parse(source: &str) -> Program {
    let mut lexer = Token::lexer(source);
    let mut program = Program::new();
    let mut pending = Pending::None;
    let mut errors: Vec<String> = Vec::new();

    while let Some(result) = lexer.next() {
        let line = line_of(source, lexer.span().start);
        let Ok(token) = result else {
            errors.push(format!(
                "line {line}: unrecognized token '{}'",
                lexer.slice()
            ));
            continue;
        };

        match token {
            Token::MetaOperator(op) => match op.as_str() {
                "@point" => pending = Pending::Point,
                "@push" => pending = Pending::PushMulti,
                "@pushstr" => pending = Pending::PushStr,
                other => errors.push(format!("line {line}: unknown meta operator '{other}'")),
            },

            Token::Operator(op) => match op.as_str() {
                "add" => {
                    program.push(Node::Instr(Instr::Add, line));
                    pending = Pending::None;
                }
                "sub" => {
                    program.push(Node::Instr(Instr::Sub, line));
                    pending = Pending::None;
                }
                "dup" => {
                    program.push(Node::Instr(Instr::Dup, line));
                    pending = Pending::None;
                }
                "swap" => {
                    program.push(Node::Instr(Instr::Swap, line));
                    pending = Pending::None;
                }
                "print" => {
                    pending = Pending::Print;
                }
                "ret" => {
                    program.push(Node::Instr(Instr::Ret, line));
                    pending = Pending::None;
                }
                "pop" => {
                    program.push(Node::Instr(Instr::Pop, line));
                    pending = Pending::None;
                }
                "mul" => {
                    program.push(Node::Instr(Instr::Mul, line));
                    pending = Pending::None;
                }
                "mod" => {
                    program.push(Node::Instr(Instr::Mod, line));
                    pending = Pending::None
                }
                "div" => {
                    program.push(Node::Instr(Instr::Div, line));
                    pending = Pending::None
                }
                "load" => {
                    program.push(Node::Instr(Instr::Load, line));
                    pending = Pending::None
                }
                "store" => {
                    program.push(Node::Instr(Instr::Store, line));
                    pending = Pending::None
                }
                "eq" => {
                    program.push(Node::Instr(Instr::Eq, line));
                    pending = Pending::None;
                }
                "lt" => {
                    program.push(Node::Instr(Instr::Lt, line));
                    pending = Pending::None;
                }
                "gte" => {
                    program.push(Node::Instr(Instr::Gte, line));
                    pending = Pending::None;
                }
                "push" => pending = Pending::Push,
                "jmp" => pending = Pending::Jmp,
                "jz" => pending = Pending::Jz,
                ident => match pending {
                    Pending::Jmp => {
                        program.push(Node::Instr(Instr::Jmp(ident.to_string()), line));
                        pending = Pending::None;
                    }
                    Pending::Jz => {
                        program.push(Node::Instr(Instr::Jz(ident.to_string()), line));
                        pending = Pending::None;
                    }
                    Pending::Point => {
                        program.push(Node::Label(ident.to_string(), line));
                        pending = Pending::None;
                    }
                    _ => errors.push(format!("line {line}: unexpected identifier '{ident}'")),
                },
            },

            Token::Number(n)
                if matches!(pending, Pending::Push | Pending::PushMulti | Pending::Print) =>
            {
                match n.parse::<u8>() {
                    Ok(value) => {
                        if matches!(pending, Pending::Print) {
                            program.push(Node::Instr(Instr::Print(value), line));
                            pending = Pending::None;
                            continue;
                        }
                        program.push(Node::Instr(Instr::Push(value), line));
                        if matches!(pending, Pending::Push) {
                            pending = Pending::None;
                        }
                    }
                    Err(_) => errors.push(format!(
                        "line {line}: invalid number literal '{n}' (must fit in u8)"
                    )),
                }
            }

            Token::Text(text) if matches!(pending, Pending::PushStr) => {
                for byte in text.bytes() {
                    program.push(Node::Instr(Instr::Push(byte), line));
                }
            }

            Token::Semicolon => pending = Pending::None,

            _ => {}
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}: {}", "Error".red(), e.yellow());
        }
        exit(11);
    }

    program
}

pub fn resolve_labels(program: &Program) -> HashMap<String, u16> {
    let mut labels = HashMap::new();
    let mut offset: u16 = 0;
    for node in program {
        match node {
            Node::Label(name, _line) => {
                labels.insert(name.clone(), offset);
            }
            Node::Instr(instr, _line) => offset += instr.size(),
        }
    }
    labels
}

pub fn codegen(program: &Program, labels: &HashMap<String, u16>) -> Vec<u8> {
    let mut bytecode = Vec::new();

    for node in program {
        let (instr, line) = match node {
            Node::Label(..) => continue,
            Node::Instr(i, line) => (i, *line),
        };

        match instr {
            Instr::Add => bytecode.push(Opcode::Add as u8),
            Instr::Sub => bytecode.push(Opcode::Sub as u8),
            Instr::Dup => bytecode.push(Opcode::Dup as u8),
            Instr::Swap => bytecode.push(Opcode::Swap as u8),
            Instr::Mul => bytecode.push(Opcode::Mul as u8),
            Instr::Pop => bytecode.push(Opcode::Pop as u8),
            Instr::Div => bytecode.push(Opcode::Div as u8),
            Instr::Mod => bytecode.push(Opcode::Mod as u8),
            Instr::Load => bytecode.push(Opcode::Load as u8),
            Instr::Store => bytecode.push(Opcode::Store as u8),
            Instr::Eq => bytecode.push(Opcode::Eq as u8),
            Instr::Gte => bytecode.push(Opcode::Gte as u8),
            Instr::Lt => bytecode.push(Opcode::Lt as u8),
            Instr::Print(v) => {
                bytecode.push(Opcode::Print as u8);
                bytecode.push(*v);
            }
            Instr::Ret => bytecode.push(Opcode::Ret as u8),
            Instr::Push(v) => {
                bytecode.push(Opcode::Push as u8);
                bytecode.push(*v);
            }
            Instr::Jmp(label) => emit_jump(&mut bytecode, Opcode::Jmp, label, line, labels),
            Instr::Jz(label) => emit_jump(&mut bytecode, Opcode::Jz, label, line, labels),
        }
    }

    bytecode
}

fn emit_jump(
    bytecode: &mut Vec<u8>,
    opcode: Opcode,
    label: &str,
    line: usize,
    labels: &HashMap<String, u16>,
) {
    bytecode.push(opcode as u8);
    let addr = labels.get(label).unwrap_or_else(|| {
        eprintln!(
            "{}: line {}: label \"{}\" is not defined",
            "Error".red(),
            line,
            label.yellow()
        );
        exit(11);
    });
    let bytes = addr.to_be_bytes();
    bytecode.push(bytes[0]);
    bytecode.push(bytes[1]);
}

#[derive(Parser)]
struct Args {
    source_file: String,
    #[arg(default_value = "bin.exc")]
    output_file: String,
}

fn main() {
    let args = Args::parse();
    let content = std::fs::read_to_string(&args.source_file).unwrap();

    let program = parse(&content);
    let labels = resolve_labels(&program);
    let bytecode = codegen(&program, &labels);

    let binary = Binary {
        magic: [0xCA, 0xFE, 0xCA, 0xFE],
        version: 67,
        code: bytecode,
    };
    let bytes = postcard::to_allocvec(&binary).unwrap();
    std::fs::write(&args.output_file, bytes).unwrap();

    println!(
        "{}: binary file saved to: {}",
        "Succeful compiled".green(),
        args.output_file.yellow()
    );
}
