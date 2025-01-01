use crate::token::{LiteralType, Token};

#[derive(Debug, Clone)]
pub enum Expr {
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
