use std::{iter::Peekable, mem, str::Chars};

use crate::{
    token::{Token, TokenType},
    utils::StringUtils,
    RloxError,
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    iter: Peekable<Chars<'static>>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new(source: String) -> Self {
        // the reason for using unsafe here is to have the ability to use utf-8 symbols
        // rust doesn't allow having both the iterator and iterable inside one
        // structure(understandably so bcs of reference invalidation)
        let chars = unsafe {
            mem::transmute::<std::str::Chars<'_>, std::str::Chars<'static>>(source.chars())
        };
        Self {
            source,
            iter: chars.peekable(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    // this is so awful for me to write. This function needs to be not mutable in theory and it
    // could be accomplished. TODO!
    fn scan_tokens(&mut self) -> Result<&Vec<Token>, Vec<RloxError>> {
        let mut errors = Vec::new();
        while let Some(character) = self.advance() {
            self.start = self.current;
            let result = self.scan_token(character);
            if let Err(e) = result {
                errors.push(RloxError {
                    msg: e.to_string(),
                    line: self.line,
                });
            }
        }

        self.tokens.push(Token {
            t_type: TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(&self.tokens)
    }

    //fn is_at_end(&self) -> bool {
    //    self.current >= self.source.len()
    //}

    fn scan_token(&mut self, token: char) -> Result<(), &'static str> {
        let mut error = Ok(());

        match token {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' if self.peek_and_match('=') => self.add_token(TokenType::BangEqual),
            '!' => self.add_token(TokenType::Bang),
            '=' if self.peek_and_match('=') => self.add_token(TokenType::EqualEqual),
            '=' => self.add_token(TokenType::Equal),
            '<' if self.peek_and_match('=') => self.add_token(TokenType::LessEqual),
            '<' => self.add_token(TokenType::Less),
            '>' if self.peek_and_match('>') => self.add_token(TokenType::GreaterEqual),
            '>' => self.add_token(TokenType::Greater),
            // checking for comments and just advance the iterator
            '/' if self.peek_and_match('/') => {
                while self.peek().is_some_and(|x| x != '\n') {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash),

            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            _ => error = Err("Unexpected character"),
        };

        error
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.iter.next()
    }

    fn add_token(&mut self, t_type: TokenType) {
        self.add_token_literal(t_type, None)
    }

    fn add_token_literal(&mut self, t_type: TokenType, literal: Option<Box<dyn std::any::Any>>) {
        let text = self.source.substring(self.start, self.current);
        self.tokens.push(Token {
            t_type,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        });
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().copied()
    }

    fn peek_and_match(&mut self, expected: char) -> bool {
        let peek = self.peek();
        if peek.is_some_and(|x| x == expected) {
            self.advance();
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenType::*;

    fn do_cols_match<T: PartialEq>(a: &[T], b: &[T]) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    #[test]
    fn should_be_equal() {
        let value = r#"
            // this is a comment
            (( )){} // grouping stuff
            !*+-/=<> <= == // operators
        "#;

        let mut scanner = Scanner::new(value.to_string());

        let expected_tokens = vec![
            LeftParen, LeftParen, RightParen, RightParen, LeftBrace, RightBrace, Bang, Star, Plus,
            Minus, Slash, Equal, Less, Greater, LessEqual, EqualEqual, EOF,
        ];

        let actual_tokens: Vec<TokenType> = scanner
            .scan_tokens()
            .unwrap()
            .iter()
            .map(|x| x.t_type)
            .collect();

        assert!(do_cols_match(&actual_tokens, &expected_tokens));
    }
}
