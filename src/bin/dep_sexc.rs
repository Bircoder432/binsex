use binsex::lexer::Token::Whitespace;
use binsex::{Binary, Opcode, lexer::Token};
use clap::Parser;
use colored::Colorize;
use logos::Lexer;
use logos::Logos;
use std::{
    collections::{HashMap, HashSet},
    fs,
    process::exit,
};

#[derive(Parser)]
struct Args {
    source_file: String,
    #[arg(default_value = "bin.exc")]
    output_file: String,
}

fn main() {
    let args = Args::parse();
    let content = std::fs::read_to_string(args.source_file).unwrap();
    let mut lexer_meta = Token::lexer(&content);
    let mut lexer_compile = Token::lexer(&content);
    let mut bytecode: Vec<u8> = vec![];
    let mut pointmap: HashMap<String, u16> = HashMap::new();
    new_meta_step(&mut lexer_meta, &mut bytecode, &mut pointmap);
    compile_step(&mut lexer_compile, &mut bytecode, &mut pointmap);
    let binary = Binary {
        magic: [0xCA, 0xFE, 0xCA, 0xFE],
        version: 67,
        code: bytecode,
    };
    let bytes = postcard::to_allocvec(&binary).unwrap();
    std::fs::write(args.output_file.clone(), bytes).unwrap();
    println!(
        "{}: binary file saved to: {}",
        "Succeful compiled".green(),
        args.output_file.yellow()
    );
}

fn compile_step(
    lexer: &mut Lexer<Token>,
    bytecode: &mut Vec<u8>,
    pointmap: &mut HashMap<String, u16>,
) {
    let mut curop = "";
    while let Some(token) = lexer.next() {
        match token.unwrap() {
            Token::Operator(operator) => match operator.as_str() {
                "add" => {
                    bytecode.push(Opcode::Add as u8);
                }
                "push" => {
                    bytecode.push(Opcode::Push as u8);
                    curop = "push";
                }
                "jmp" => {
                    bytecode.push(Opcode::Jmp as u8);
                    curop = "jmp"
                }
                "jz" => {
                    bytecode.push(Opcode::Jz as u8);
                    curop = "jz"
                }
                "sub" => {
                    bytecode.push(Opcode::Sub as u8);
                }
                "dup" => {
                    bytecode.push(Opcode::Dup as u8);
                }
                "swap" => bytecode.push(Opcode::Swap as u8),
                "print" => {
                    bytecode.push(Opcode::Print as u8);
                }
                "ret" => {
                    bytecode.push(Opcode::Ret as u8);
                }
                x if pointmap.contains_key(x) => match curop {
                    "jmp" => {
                        let bytes = pointmap
                            .get(&operator)
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "{}: Point \"{}\" not exist",
                                    "Error".red(),
                                    operator.green()
                                );
                                exit(11);
                            })
                            .to_be_bytes();
                        bytecode.push(bytes[0]);
                        bytecode.push(bytes[1]);
                        curop = "";
                        continue;
                    }
                    "jz" => {
                        let bytes = pointmap
                            .get(&operator)
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "{}: Point \"{}\" not exist",
                                    "Error".red(),
                                    operator.green()
                                );
                                exit(11);
                            })
                            .to_be_bytes();
                        bytecode.push(bytes[0]);
                        bytecode.push(bytes[1]);
                        curop = "";
                        continue;
                    }
                    _ => {}
                },
                _ => {}
            },
            Token::Number(num) => match curop {
                "push" => {
                    bytecode.push(num.parse::<u8>().unwrap());
                    curop = "";
                    continue;
                }

                _ => {}
            },
            _ => {}
        }
    }
}

fn new_meta_step(
    lexer: &mut Lexer<Token>,
    bytecode: &mut Vec<u8>,
    pointmap: &mut HashMap<String, u16>,
) {
    let mut new_point = None;
    let mut opid = 0;
    let mut push = false;
    let mut push_str = false;
    let mut curop = "";
    let mut unknowns: Vec<String> = Vec::new();
    while let Some(token) = lexer.next() {
        match token.unwrap_or(Whitespace) {
            Token::MetaOperator(operator) => match operator.as_str() {
                "@point" => {
                    new_point = Some(opid);
                    curop = "fuckable";
                }
                "@push" => push = true,
                "@pushstr" => push_str = true,
                unknown => {
                    unknowns.push(unknown.to_string());
                }
            },
            Token::Operator(text) => match text.as_str() {
                "print" | "ret" | "add" | "dup" | "sub" => {
                    opid += 1;
                }
                "push" => {
                    opid += 2;
                }
                "jmp" | "jz" => {
                    curop = "fuckable";
                    opid += 3;
                }
                unknown => {
                    if curop == "fuckable" {
                        curop = "";
                        if let Some(point) = new_point {
                            pointmap.insert(unknown.to_string(), point);
                            new_point = None;
                        }
                        continue;
                    }
                    unknowns.push(unknown.to_string());
                }
            },
            Token::Number(num) => {
                if push {
                    bytecode.push(Opcode::Push as u8);
                    bytecode.push(num.parse::<u8>().unwrap());
                    opid += 1;
                }
            }
            Token::Text(text) => {
                if push_str {
                    let bytes: Vec<u8> = text.bytes().collect();

                    for byte in bytes {
                        bytecode.push(Opcode::Push as u8);
                        bytecode.push(byte);
                        opid += 2;
                    }
                }
            }
            Token::Semicolon => {
                push = false;
                push_str = false;
            }
            _ => {}
        }
    }
    if !unknowns.is_empty() {
        for unknown in unknowns {
            eprintln!("{}: Unknown operator: {}", "Error".red(), unknown.yellow(),);
        }
        exit(11);
    }
}
