use crate::{
    ast::Expr,
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

pub fn interpret(expr: &Expr) -> Result<LiteralType, RuntimeError> {
    match expr {
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => ternary(first, second, third),
        Expr::Binary { left, op, right } => binary(&interpret(left)?, &interpret(right)?, op),
        Expr::Grouping { expression } => interpret(expression),
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Unary { op, right } => Ok(unary(&interpret(right)?, op)),
    }
}

fn ternary(first: &Expr, second: &Expr, third: &Expr) -> Result<LiteralType, RuntimeError> {
    let first = interpret(first)?;
    if is_truthy(&first) {
        return interpret(second);
    }
    interpret(third)
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
        (Slash, Number(left), Number(right)) => Ok(Number(left / right)),
        (Star, Number(left), Number(right)) => Ok(Number(left * right)),
        (Comma, _,_) => Ok(right.clone()),
        (Greater | GreaterEqual | Less | LessEqual | Minus | Slash | Star, _, _) => Err(RuntimeError::new(op, "Operands must be numbers")),
        (Plus, _, _) => Err(RuntimeError::new(op, "Operands must be two numbers or two strings")),
        /* comma operator discard the left operand, so we just return the evaluation of the right operand */

        _ => unreachable!("Shouldn't happen. Expr::Binary for interpret. Some case is a binary operation that wasn't matched")
    }
}

fn unary(right: &LiteralType, op: &Token) -> LiteralType {
    match (op.t_type, &right) {
        (TokenType::Minus, LiteralType::Number(num)) => LiteralType::Number(-num),
        (TokenType::Bang, _) => LiteralType::Bool(!is_truthy(right)),
        _ => unreachable!("Shouldn't happen. Expr::Unary for interpret"),
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
