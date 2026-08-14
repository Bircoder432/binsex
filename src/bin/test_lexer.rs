use std::fs;

use binsex::lexer::Token;
use logos::Logos;

fn main() {
    let bytes = "End".as_bytes();
    for byte in bytes {
        println!("push {}", byte);
    }
    println!("Size: {}", bytes.len());
}
