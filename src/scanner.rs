use std::{iter::Peekable, mem, str::Chars};

use crate::{
    token::{LiteralType, Token, TokenType},
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
        while self.peek().is_some() {
            self.start = self.current;
            let result = self.scan_token();
            if let Err(e) = result {
                errors.push(e);
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

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), RloxError> {
        let token = self.advance().unwrap();
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
            // checking for comments and just advance the iterator if it's a comment
            '/' if self.peek_and_match('/') => {
                while self.peek().is_some_and(|x| x != '\n') {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash),
            '"' => error = self.string(),
            '0'..='9' => self.number(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            _ => {
                error = Err(RloxError {
                    msg: "Unexpected character".to_string(),
                    line: self.line,
                })
            }
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

    fn add_token_literal(&mut self, t_type: TokenType, literal: Option<LiteralType>) {
        let text = self.source.slice(self.start..self.current);
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

    fn peek_double(&mut self) -> Option<char> {
        let mut copied_iterator = self.iter.clone();
        copied_iterator.next();
        copied_iterator.peek().copied()
    }

    fn peek_and_match(&mut self, expected: char) -> bool {
        let peek = self.peek();
        if peek.is_some_and(|x| x == expected) {
            self.advance();
            return true;
        }

        false
    }

    fn string(&mut self) -> Result<(), RloxError> {
        let start_line = self.line;
        while self.peek().is_some_and(|x| x != '"') {
            if self.peek().is_some_and(|x| x == '\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.peek().is_none() {
            let error = RloxError {
                msg: "Unterminated string".to_string(),
                line: start_line,
            };
            return Err(error);
        }

        self.advance();

        // clean out the quotes and wrap it in a string literal type
        let value = LiteralType::String(
            self.source
                .slice(self.start + 1..self.current - 1)
                .to_string(),
        );

        self.add_token_literal(TokenType::String, Some(value));

        Ok(())
    }

    fn number(&mut self) {
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }

        if self.peek().is_some_and(|x| x == '.') && matches!(self.peek_double(), Some('0'..='9')) {
            self.advance();

            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }

        let number: f64 = self
            .source
            .slice(self.start..self.current)
            .parse()
            .expect("There shouldn't be any errors. Please check");

        self.add_token_literal(TokenType::Number, Some(LiteralType::Number(number)));
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

    #[test]
    fn correct_string_scan() {
        let value = r#"
            // string!
            "salam!""#;

        let mut scanner = Scanner::new(value.to_string());

        let tokens: Vec<&Token> = scanner
            .scan_tokens()
            .expect("Should not be an error!")
            .iter()
            .filter(|x| matches!(x.t_type, TokenType::String))
            .collect();

        let actual = tokens[0];

        let expected = LiteralType::String("salam!".to_string());

        assert_eq!(expected, actual.literal.as_ref().unwrap().clone());
    }

    #[test]
    fn error_string_scan() {
        let value = r#"
            // Unterminated string bro wtf
            "salam

            (){} {}"#
            .to_string();

        let mut scanner = Scanner::new(value);

        let expected_error = RloxError {
            msg: "Unterminated string".to_string(),
            line: 3,
        };

        let tokens = scanner.scan_tokens().expect_err("Should be an error");

        let actual_error = tokens
            .iter()
            .find(|e| e.msg == "Unterminated string")
            .expect("Error not found. There should be an error");

        assert_eq!(expected_error, actual_error.clone());
    }

    #[test]
    fn correct_whole_number_scan() {
        let value = r#"
            // number test
            123"#
            .to_string();

        let mut scanner = Scanner::new(value);

        let expected_value = LiteralType::Number(123.0);

        let tokens = scanner.scan_tokens().expect("There shouldn't be an error");

        let token = tokens
            .iter()
            .find(|t| matches!(t.t_type, TokenType::Number))
            .expect("There should be a number here. Couldn't find it");

        let actual_value = &token.literal;

        assert_eq!(expected_value, actual_value.as_ref().unwrap().clone())
    }

    #[test]
    fn correct_fractional_number_scan() {
        let value = r#"
            // number test
            123.456"#
            .to_string();

        let mut scanner = Scanner::new(value);

        let expected_value = LiteralType::Number(123.456);

        let tokens = scanner.scan_tokens().expect("There shouldn't be an error");

        let token = tokens
            .iter()
            .find(|t| matches!(t.t_type, TokenType::Number))
            .expect("There should be a number here. Couldn't find it");

        let actual_value = &token.literal;

        assert_eq!(expected_value, actual_value.as_ref().unwrap().clone())
    }
}
