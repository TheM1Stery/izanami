use core::panic;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{Expr, Stmt},
    callable::{Callable, CallableTrait, NativeFunction},
    environment::Environment,
    token::{LiteralType, Token, TokenType},
};

type InterpreterResult = Result<LiteralType, InterpreterSignal>;

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

pub struct InterpreterEnvironment {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
}

impl RuntimeError {
    pub fn new(token: &Token, message: String) -> Self {
        RuntimeError {
            token: Some(token.clone()),
            message: message.to_string(),
        }
    }

    pub fn no_token(message: String) -> Self {
        RuntimeError {
            token: None,
            message: message.to_string(),
        }
    }
}

pub enum InterpreterSignal {
    RuntimeError(RuntimeError),
    Break,
    Return(LiteralType),
}
/*
    This two impl blocks are for the ? operator. I'm too lazy to write the wrapping code for the enums and it also looks ugly,
    so i just abuse the ? operator lol
    Instead of InterpreterError::RuntimeError(RuntimeError {...} ) i can just RuntimeError {...}? to turn it into a InterpreterError
*/

impl From<RuntimeError> for InterpreterSignal {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

impl From<InterpreterSignal> for RuntimeError {
    fn from(value: InterpreterSignal) -> Self {
        match value {
            InterpreterSignal::RuntimeError(runtime_error) => runtime_error,
            InterpreterSignal::Break => panic!("Not a runtime error"),
            InterpreterSignal::Return(_) => panic!("Not a runtime error"),
        }
    }
}

pub fn interpret(
    statements: &Vec<Stmt>,
    environment: &Rc<RefCell<Environment>>,
) -> Result<(), InterpreterSignal> {
    let clock = |_arg: &[LiteralType]| {
        Ok(LiteralType::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64()
                / 1000.0,
        ))
    };

    let clock_function = NativeFunction::new("clock".to_string(), 0, clock);
    let environment = InterpreterEnvironment {
        globals: Rc::clone(environment),
        environment: Rc::clone(environment),
    };
    environment.globals.borrow_mut().define(
        "clock",
        Some(LiteralType::Callable(Callable::NativeFunction(
            clock_function,
        ))),
    );
    environment.globals.borrow_mut().define(
        "read_input",
        Some(LiteralType::Callable(Callable::NativeFunction(
            read_input_function(),
        ))),
    );
    for statement in statements {
        execute(statement, &environment)?
    }

    Ok(())
}

fn execute(
    statement: &Stmt,
    environment: &InterpreterEnvironment,
) -> Result<(), InterpreterSignal> {
    let curr_environment = &environment.environment;
    match statement {
        Stmt::Expression { expression } => {
            evaluate(expression, environment)?;
        }
        Stmt::Print { expression } => {
            let expr = evaluate(expression, environment)?;
            println!("{expr}");
        }
        Stmt::Var { name, initializer } => {
            let value = if let Some(initializer) = initializer {
                Some(evaluate(initializer, environment)?)
            } else {
                None
            };
            curr_environment.borrow_mut().define(&name.lexeme, value);
        }
        Stmt::Block { statements } => {
            execute_block(statements, environment)?;
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if is_truthy(&evaluate(condition, environment)?) {
                execute(then_branch, environment)?;
            } else if let Some(else_branch) = else_branch {
                execute(else_branch, environment)?;
            }
        }
        Stmt::While { condition, body } => {
            while is_truthy(&evaluate(condition, environment)?) {
                let result = execute(body, environment);
                if result.is_err() {
                    break;
                }
            }
        }
        Stmt::Break => Err(InterpreterSignal::Break)?,
        Stmt::Function { name, params, body } => {
            let function = Callable::Function {
                name: Box::new(name.clone()),
                body: body.to_vec(),
                params: params.to_vec(),
                closure: Rc::clone(curr_environment),
            };
            environment
                .globals
                .borrow_mut()
                .define(&name.lexeme, Some(LiteralType::Callable(function)));
        }
        Stmt::Return { value, .. } => {
            let value = if let Some(v) = value {
                evaluate(v, environment)?
            } else {
                LiteralType::Nil
            };

            return Err(InterpreterSignal::Return(value));
        }
    }

    Ok(())
}

pub fn execute_block(
    statements: &Vec<Stmt>,
    environment: &InterpreterEnvironment,
) -> Result<(), InterpreterSignal> {
    let block_enviroment = Rc::new(RefCell::new(Environment::with_enclosing(
        &environment.environment,
    )));
    // we just move the block_enviroment to a new InterpreterEnvironment and clone the reference to
    // globals, bcs outer environments might have the globals reference
    let environment = InterpreterEnvironment {
        globals: Rc::clone(&environment.globals),
        environment: block_enviroment,
    };
    for stmt in statements {
        execute(stmt, &environment)?;
    }

    Ok(())
}

fn evaluate(expr: &Expr, environment: &InterpreterEnvironment) -> InterpreterResult {
    let curr_environment = &environment.environment;
    match expr {
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => ternary(first, second, third, environment),
        Expr::Binary { left, op, right } => binary(
            &evaluate(left, environment)?,
            &evaluate(right, environment)?,
            op,
        ),
        Expr::Grouping { expression } => evaluate(expression, environment),
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Unary { op, right } => Ok(unary(&evaluate(right, environment)?, op)),
        Expr::Variable { name } => curr_environment
            .borrow()
            .get(name)
            .ok_or_else(|| RuntimeError::new(name, format!("Undefined variable {}.", name.lexeme)))
            .and_then(|x| {
                x.ok_or_else(|| {
                    RuntimeError::new(name, format!("Uninitialized variable {}.", name.lexeme))
                })
            })
            .map_err(InterpreterSignal::RuntimeError),
        Expr::Assign { name, value } => {
            let value = evaluate(value, environment)?;
            curr_environment
                .borrow_mut()
                .assign(name, value.clone())
                .map_err(|_| {
                    RuntimeError::new(name, format!("Undefined variable {}.", name.lexeme))
                })?;
            Ok(value)
        }
        Expr::Logical { left, op, right } => {
            let left = evaluate(left, environment)?;

            if op.t_type == TokenType::OR {
                if is_truthy(&left) {
                    return Ok(left);
                }
            } else if !is_truthy(&left) {
                return Ok(left);
            }

            evaluate(right, environment)
        }
        Expr::Call {
            callee,
            paren,
            args,
        } => {
            let callee_result = evaluate(callee, environment)?;

            let mut arguments = Vec::new();
            for arg in args {
                arguments.push(evaluate(arg, environment)?);
            }

            match callee_result {
                LiteralType::Callable(function) => {
                    if arguments.len() as u8 != function.arity() {
                        Err(RuntimeError::new(
                            paren,
                            format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                args.len()
                            ),
                        ))?
                    }
                    Ok(function.call(&arguments, environment)?)
                }
                _ => Err(RuntimeError::new(
                    paren,
                    "Can only call functions and classes".to_string(),
                ))?,
            }
        }
    }
}

fn ternary(
    first: &Expr,
    second: &Expr,
    third: &Expr,
    environment: &InterpreterEnvironment,
) -> InterpreterResult {
    let first = evaluate(first, environment)?;
    if is_truthy(&first) {
        return evaluate(second, environment);
    }
    evaluate(third, environment)
}

fn binary(left: &LiteralType, right: &LiteralType, op: &Token) -> InterpreterResult {
    use LiteralType::{Bool, Number, String};
    use TokenType::{
        BangEqual, Comma, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, Plus, Slash,
        Star,
    };

    match (op.t_type, &left, &right) {
        (Greater, Number(left), Number(right)) => Ok(Bool(left > right)),
        (GreaterEqual, Number(left), Number(right)) => Ok(Bool(left >= right)),
        (Less, Number(left), Number(right)) => Ok(Bool(left < right)),
        (LessEqual, Number(left), Number(right)) => Ok(Bool(left <= right)),
        (BangEqual, _, _) => Ok(Bool(!is_equal(left, right))),
        (EqualEqual, _, _) => Ok(Bool(is_equal(left, right))),
        (Minus, Number(left), Number(right)) => Ok(Number(left - right)),
        (Plus, Number(left), Number(right)) => Ok(Number(left + right)),
        (Plus, String(left), String(right)) => Ok(String(format!("{left}{right}"))),
        (Plus, String(left), Number(right)) => Ok(String(format!("{left}{right}"))),
        (Plus, Number(left), String(right)) => Ok(String(format!("{left}{right}"))),
        (Slash, Number(left), Number(right)) => Ok(Number(left / right)),
        (Star, Number(left), Number(right)) => Ok(Number(left * right)),
        /* comma operator discard the left operand, so we just return the evaluation of the right operand */
        (Comma, _,_) => Ok(right.clone()),
        (Greater | GreaterEqual | Less | LessEqual | Minus | Slash | Star, _, _) => Err(RuntimeError::new(op, "Operands must be numbers".to_string()))?,
        (Plus, _, _) => Err(RuntimeError::new(op, "Operands must be two numbers or two strings".to_string()))?,

        _ => unreachable!("Shouldn't happen. Expr::Binary for evaluate. Some case is a binary operation that wasn't matched")
    }
}

fn unary(right: &LiteralType, op: &Token) -> LiteralType {
    match (op.t_type, &right) {
        (TokenType::Minus, LiteralType::Number(num)) => LiteralType::Number(-num),
        (TokenType::Bang, _) => LiteralType::Bool(!is_truthy(right)),
        _ => unreachable!("Shouldn't happen. Expr::Unary for evaluate"),
    }
}

fn is_truthy(literal: &LiteralType) -> bool {
    match literal {
        LiteralType::Nil => false,
        LiteralType::Bool(val) => *val,
        _ => true,
    }
}

pub fn is_equal(left: &LiteralType, right: &LiteralType) -> bool {
    match (left, right) {
        (LiteralType::Nil, LiteralType::Nil) => true,
        (LiteralType::Nil, _) => false,
        // i could've implemeneted PartialEq but it doesn't make sense for every LiteralType
        (LiteralType::String(s), LiteralType::String(s2)) => s == s2,
        (LiteralType::Number(n1), LiteralType::Number(n2)) => n1 == n2,
        (LiteralType::Bool(t1), LiteralType::Bool(t2)) => t1 == t2,
        _ => false,
    }
}

fn read_input_function() -> NativeFunction {
    use std::io;
    let read_input = |_: &[LiteralType]| {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .map_err(|_| RuntimeError::no_token("Error reading from stdin".to_string()))?;

        Ok(LiteralType::String(buf))
    };

    NativeFunction::new("read_input".to_string(), 0, read_input)
}
