use std::{
    error::Error,
    fs,
    io::{self, Write},
};

use interpreter::RuntimeError;
use parser::Parser;
use scanner::Scanner;
use token::{Token, TokenType};

mod ast;
mod interpreter;
mod parser;
mod printer;
mod scanner;
mod token;
mod utils;

#[derive(Debug)]
pub enum RunError {
    FileReadError(io::Error),
    OtherError(Box<dyn Error>), // to be added,
    RuntimeError(RuntimeError),
}

impl<E: Error + 'static> From<E> for RunError {
    fn from(value: E) -> Self {
        Self::OtherError(Box::new(value))
    }
}

pub fn run_file(path: &str) -> Result<(), RunError> {
    let file = fs::read_to_string(path).map_err(RunError::FileReadError)?;

    run(&file)?;
    Ok(())
}

pub fn run(src: &str) -> Result<(), RunError> {
    let mut scanner = Scanner::new(src.to_string());
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);

    let expression = parser.parse().inspect_err(|e| error(&e.token, &e.msg))?;

    let interpreted_value = interpreter::interpret(&expression)
        .inspect_err(runtime_error)
        .map_err(RunError::RuntimeError)?;

    println!("{interpreted_value}");

    Ok(())
}

pub fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin.read_line(input)?;
        let _ = run(input);
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

fn error(token: &Token, message: &str) {
    match token.t_type {
        TokenType::EOF => report(token.line, " at end", message),
        _ => report(token.line, &format!(" at '{}'", token.lexeme), message),
    }
}

fn runtime_error(err: &RuntimeError) {
    eprintln!("{}\n[line {}]", err.message, err.token.line);
}
