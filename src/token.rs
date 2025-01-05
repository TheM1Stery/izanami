use std::fmt::Display;

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
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl LiteralType {
    pub fn string_literal(val: &str) -> LiteralType {
        LiteralType::String(val.to_string())
    }

    pub fn number_literal(val: f64) -> LiteralType {
        LiteralType::Number(val)
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
