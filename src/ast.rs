use crate::token::{LiteralType, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Ternary {
        first: Box<Expr>,
        second: Box<Expr>,
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
        value: LiteralType,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
}
