use crate::token::{LiteralType, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Ternary {
        first: Box<Expr>,
        first_op: Token,
        second: Box<Expr>,
        second_op: Token,
        third: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Option<LiteralType>,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
}
