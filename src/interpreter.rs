use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::{LiteralType, Token, TokenType},
};

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> RuntimeError {
        RuntimeError {
            token: token.clone(),
            message: message.to_string(),
        }
    }
}

pub fn interpret(
    statements: &Vec<Stmt>,
    environment: &Rc<RefCell<Environment>>,
) -> Result<(), RuntimeError> {
    for statement in statements {
        execute(statement, environment)?;
    }

    Ok(())
}

fn execute(statement: &Stmt, environment: &Rc<RefCell<Environment>>) -> Result<(), RuntimeError> {
    match statement {
        Stmt::Expression { expression } => {
            evaluate(expression, &mut environment.borrow_mut())?;
        }
        Stmt::Print { expression } => {
            let expr = evaluate(expression, &mut environment.borrow_mut())?;
            println!("{expr}");
        }
        Stmt::Var { name, initializer } => {
            let value = if let Some(initializer) = initializer {
                Some(evaluate(initializer, &mut environment.borrow_mut())?)
            } else {
                None
            };
            environment.borrow_mut().define(&name.lexeme, value);
        }
        Stmt::Block { statements } => {
            execute_block(statements, environment)?;
        }
    }

    Ok(())
}

fn execute_block(
    statements: &Vec<Stmt>,
    environment: &Rc<RefCell<Environment>>,
) -> Result<(), RuntimeError> {
    let block_enviroment = Rc::new(RefCell::new(Environment::with_enclosing(environment)));
    for stmt in statements {
        execute(stmt, &block_enviroment)?;
    }

    Ok(())
}

fn evaluate(expr: &Expr, environment: &mut Environment) -> Result<LiteralType, RuntimeError> {
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
        Expr::Variable { name } => environment
            .get(name)
            .ok_or_else(|| RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable {}.", name.lexeme),
            })
            .and_then(|x| {
                x.ok_or_else(|| RuntimeError {
                    token: name.clone(),
                    message: format!("Uninitialized variable {}.", name.lexeme),
                })
            }),
        Expr::Assign { name, value } => {
            let value = evaluate(value, environment)?;
            environment
                .assign(name, value.clone())
                .map_err(|_| RuntimeError {
                    token: name.clone(),
                    message: format!("Undefined variable {}.", name.lexeme),
                })?;

            Ok(value)
        }
    }
}

fn ternary(
    first: &Expr,
    second: &Expr,
    third: &Expr,
    environment: &mut Environment,
) -> Result<LiteralType, RuntimeError> {
    let first = evaluate(first, environment)?;
    if is_truthy(&first) {
        return evaluate(second, environment);
    }
    evaluate(third, environment)
}

fn binary(
    left: &LiteralType,
    right: &LiteralType,
    op: &Token,
) -> Result<LiteralType, RuntimeError> {
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
        (Greater | GreaterEqual | Less | LessEqual | Minus | Slash | Star, _, _) => Err(RuntimeError::new(op, "Operands must be numbers")),
        (Plus, _, _) => Err(RuntimeError::new(op, "Operands must be two numbers or two strings")),

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

fn is_equal(left: &LiteralType, right: &LiteralType) -> bool {
    match (left, right) {
        (LiteralType::Nil, LiteralType::Nil) => true,
        (LiteralType::Nil, _) => false,
        _ => left == right,
    }
}
