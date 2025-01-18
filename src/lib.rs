use std::{
    cell::RefCell,
    error::Error,
    fs,
    io::{self, Write},
    rc::Rc,
};

use environment::Environment;
use interpreter::RuntimeError;
use parser::{ParseError, Parser};
use scanner::Scanner;
use token::TokenType;

mod ast;
mod environment;
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
    ParseError,
}

impl<E: Error + 'static> From<E> for RunError {
    fn from(value: E) -> Self {
        Self::OtherError(Box::new(value))
    }
}

pub fn run_file(path: &str) -> Result<(), RunError> {
    let file = fs::read_to_string(path).map_err(RunError::FileReadError)?;
    let environment = Rc::new(RefCell::new(Environment::new()));

    run(&file, &environment)?;
    Ok(())
}

pub fn run(src: &str, environment: &Rc<RefCell<Environment>>) -> Result<(), RunError> {
    let mut scanner = Scanner::new(src.to_string());
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);

    let statements = parser.parse();

    // i don't want to collect the errors and allocate a vec
    let mut p_error = false;

    for err in statements.iter().filter_map(|x| x.as_ref().err()) {
        if !p_error {
            p_error = true;
        }
        error(err);
    }

    if p_error {
        return Err(RunError::ParseError);
    }

    let statements = statements.into_iter().flatten().collect();

    interpreter::interpret(&statements, environment)
        .map_err(|x| x.into())
        .inspect_err(runtime_error)
        .map_err(RunError::RuntimeError)?;

    Ok(())
}

pub fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    let environment = Rc::new(RefCell::new(Environment::new()));
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;
        stdin.read_line(input)?;
        let _ = run(input, &environment);
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

fn error(ParseError { token, msg }: &ParseError) {
    match token.t_type {
        TokenType::EOF => report(token.line, " at end", msg),
        _ => report(token.line, &format!("at '{}'", token.lexeme), msg),
    }
}

fn runtime_error(err: &RuntimeError) {
    eprintln!("{}\n[line {}]", err.message, err.token.line);
}
