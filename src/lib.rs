use std::{
    error::Error,
    fmt::Display,
    fs,
    io::{self, Write},
};

use parser::{ParseError, Parser};
use printer::pretty_print;
use scanner::Scanner;
use token::{Token, TokenType};

mod ast;
mod parser;
mod printer;
mod scanner;
mod token;
mod utils;

pub fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::read_to_string(path)?;

    run(&file)?;
    Ok(())
}

pub fn run(src: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(src.to_string());
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);

    let expression = parser.parse();

    match expression {
        Ok(expr) => println!("{}", pretty_print(&expr)),
        Err(e) => {
            error(e.token, &e.msg);
        }
    }

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

fn error(token: Token, message: &str) {
    match token.t_type {
        TokenType::EOF => report(token.line, " at end", message),
        _ => report(token.line, &format!(" at '{}'", token.lexeme), message),
    }
}
