use std::{
    error::Error,
    fs,
    io::{self, Write},
};

use token::Token;

mod scanner;
mod token;
mod utils;

pub fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read_to_string(path)?;

    Ok(())
}

pub fn run(src: &str) {
    let tokens: Vec<Token> = Vec::new();
}

pub fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    print!("> ");
    io::stdout().flush()?;
    loop {
        input.clear();
        let _ = stdin.read_line(input)?;

        print!("> ");
        io::stdout().flush()?;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RloxError {
    msg: String,
    line: usize,
}

impl RloxError {
    pub fn error(line: i32, message: &str) {
        report(line, "", message);
    }
}

pub fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}
