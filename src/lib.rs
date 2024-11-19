use std::{
    error::Error,
    fs,
    io::{self, Write},
};

use scanner::Scanner;
use token::Token;

mod scanner;
mod token;
mod utils;

pub fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read_to_string(path)?;

    run(&file);
    Ok(())
}

pub fn run(src: &str) {
    let mut scanner = Scanner::new(src.to_string());
    let tokens = scanner.scan_tokens();

    match tokens {
        Err(ref errors) => {
            for err in errors {
                report(err.line, "", &err.msg);
            }
        }
        Ok(_) => {
            for token in tokens.unwrap() {
                println!("{}", token);
            }
        }
    }
}

pub fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin.read_line(input)?;
        run(input);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RloxError {
    msg: String,
    line: usize,
}

pub fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}
