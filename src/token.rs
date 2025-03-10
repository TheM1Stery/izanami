use std::fmt::Display;

use crate::callable::Callable;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Question,
    Colon,

    Identifier,
    String,
    Number,

    And,
    Break,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    OR,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

// i've seen this implementation in the wild
#[derive(Debug, Clone)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(Callable),
}

impl LiteralType {
    pub fn string_literal(val: &str) -> LiteralType {
        LiteralType::String(val.to_string())
    }

    pub fn number_literal(val: f64) -> LiteralType {
        LiteralType::Number(val)
    }
}

impl Display for LiteralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralType::String(v) => write!(f, "{v}"),
            LiteralType::Number(v) => write!(f, "{v:.2}"),
            LiteralType::Bool(v) => write!(f, "{v}"),
            LiteralType::Nil => write!(f, "nil"),
            LiteralType::Callable(c) => write!(f, "<fn {c}>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub t_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralType>,
    pub line: usize,
}

impl Token {
    pub fn new(t_type: TokenType, lexeme: &str, literal: Option<LiteralType>, line: usize) -> Self {
        let lexeme = lexeme.to_string();
        Self {
            t_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {:?}", self.t_type, self.lexeme, self.literal)
    }
}
