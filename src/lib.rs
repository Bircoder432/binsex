use std::{io::Write, process::exit};
pub mod lexer;
use colored::*;
use serde::{Deserialize, Serialize};
#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    Add = 0x01,
    Push = 0x02,
    Jmp = 0x03,
    Jz = 0x04,
    Sub = 0x05,
    Dup = 0x06,
    Swap = 0x07,
    Pop = 0x08,
    Mul = 0x09,
    Div = 0x0a,
    Mod = 0x0b,
    Store = 0x0c,
    Load = 0x0d,
    Lt = 0x0e,
    Gte = 0x0f,
    Print = 0x10,
    Eq = 0x11,
    Ret = 0xff,
}

impl TryFrom<u8> for Opcode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Add),
            0x02 => Ok(Self::Push),
            0x03 => Ok(Self::Jmp),
            0x04 => Ok(Self::Jz),
            0x05 => Ok(Self::Sub),
            0x06 => Ok(Self::Dup),
            0x07 => Ok(Self::Swap),
            0x08 => Ok(Self::Pop),
            0x09 => Ok(Self::Mul),
            0x0a => Ok(Self::Div),
            0x0b => Ok(Self::Mod),
            0x0c => Ok(Self::Store),
            0x0d => Ok(Self::Load),
            0x0e => Ok(Self::Lt),
            0x0f => Ok(Self::Gte),
            0x10 => Ok(Self::Print),
            0x11 => Ok(Self::Eq),
            0xff => Ok(Self::Ret),
            _ => {
                eprintln!("{}: Unknown opcode: {}", "Error".red(), value);
                return Err(());
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Binary {
    pub magic: [u8; 4],
    pub version: u16,
    pub code: Vec<u8>,
}

pub fn execute(binary: Binary, debug: bool) {
    let mut stack: Vec<u8> = Vec::new();
    let mut ip = 0;
    let mut memory = vec![0u8; 65536];
    while ip < binary.code.len() {
        let opcode = binary.code[ip];
        if debug {
            println!(
                "{}: {} {:?}: {} {:?}",
                "Debug".blue(),
                ip,
                Opcode::try_from(opcode).unwrap(),
                opcode,
                stack
            );
        }
        match Opcode::try_from(opcode).unwrap() {
            Opcode::Print => {
                let size = binary.code[ip + 1];
                let mut string_bytes: Vec<u8> = vec![];
                for _ in 0..size {
                    let byte = stack.pop().unwrap();
                    string_bytes.push(byte);
                }
                string_bytes.reverse();
                let string = String::from_utf8(string_bytes).unwrap();
                print!("{}", string);
                std::io::stdout().flush().unwrap();
                ip += 1;
            }
            Opcode::Add => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            }
            Opcode::Push => {
                stack.push(binary.code[ip + 1]);
                ip += 1;
            }
            Opcode::Jmp => {
                let bytes: [u8; 2] = [binary.code[ip + 1], binary.code[ip + 2]];
                ip = u16::from_be_bytes(bytes) as usize;
                continue;
            }
            Opcode::Jz => {
                if stack.pop().unwrap() == 0 {
                    let bytes: [u8; 2] = [binary.code[ip + 1], binary.code[ip + 2]];
                    ip = u16::from_be_bytes(bytes) as usize;
                    continue;
                } else {
                    ip += 2;
                }
            }
            Opcode::Sub => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b - a);
            }
            Opcode::Dup => stack.push(*stack.last().unwrap()),
            Opcode::Swap => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
            }
            Opcode::Ret => {
                let code = stack.pop().unwrap();
                exit(code.into());
            }
            Opcode::Pop => {
                let _ = stack.pop().unwrap();
            }
            Opcode::Mul => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a * b);
            }
            Opcode::Div => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b / a);
            }
            Opcode::Mod => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b % a);
            }
            Opcode::Store => {
                let address = stack.pop().unwrap();
                let value = stack.pop().unwrap();
                memory[address as usize] = value;
            }
            Opcode::Load => {
                let address = stack.pop().unwrap();
                stack.push(memory[address as usize]);
            }
            Opcode::Eq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(if a == b { 1 } else { 0 });
            }
            Opcode::Lt => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(if b < a { 1 } else { 0 });
            }
            Opcode::Gte => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(if b >= a { 1 } else { 0 });
            }
        }

        ip += 1;
    }
}
