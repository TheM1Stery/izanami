use std::fmt::Display;

use crate::{
    ast::Expr,
    token::{LiteralType, Token, TokenType},
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub msg: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {} {}", self.token, self.msg)
    }
}

impl std::error::Error for ParseError {}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser<'_> {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.comma()
    }

    // Challenge #1. We're writing comma before equality, because it has the lowest precedence
    // comma -> equality ("," equality)* ;   // expression grammar
    fn comma(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[Comma], Self::ternary)
    }

    // ternary -> equality ("?" expression : ternary)? // expression grammar
    fn ternary(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        let expr = self.equality()?;

        if self.match_token(&[Question]) {
            let second = self.expression()?;
            let _ = self.consume(Colon, "Expected : after ternary operator ?")?;
            let third = self.ternary()?;
            return Ok(Expr::Ternary {
                first: Box::new(expr),
                second: Box::new(second),
                third: Box::new(third),
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[BangEqual, EqualEqual], Self::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[Greater, GreaterEqual, Less, LessEqual], Self::term)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[Minus, Plus], Self::factor)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[Slash, Star], Self::unary)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        if self.match_token(&[Bang, Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    /* error boundaries:
      ("!=" | "==") equality
    | (">" | ">=" | "<" | "<=") comparison
    | ("+") term
    | ("/" | "*") factor ; */
    fn primary(&mut self) -> Result<Expr, ParseError> {
        use LiteralType::*;
        use TokenType::*;

        fn create_literal(l_type: LiteralType) -> Expr {
            Expr::Literal { value: l_type }
        }

        if self.match_token(&[False]) {
            return Ok(create_literal(Bool(false)));
        }

        if self.match_token(&[True]) {
            return Ok(create_literal(Bool(true)));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(create_literal(
                self.previous()
                    .literal
                    .clone()
                    .expect("The number and string token should have a literal"),
            ));
        }

        // i included the enum name bcs of ambiguity of LiteralType and TokenType
        if self.match_token(&[TokenType::Nil]) {
            return Ok(create_literal(LiteralType::Nil));
        }

        if self.match_token(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.match_token(&[Equal, BangEqual]) {
            let _ = self.equality();
            return Err(ParseError {
                token: self.previous().clone(),
                msg: "Missing left-hand operand.".to_string(),
            });
        }

        if self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let _ = self.comparison();
            return Err(ParseError {
                token: self.previous().clone(),
                msg: "Missing left-hand operand.".to_string(),
            });
        }

        if self.match_token(&[Plus]) {
            let _ = self.term();
            return Err(ParseError {
                token: self.previous().clone(),
                msg: "Missing left-hand operand.".to_string(),
            });
        }

        if self.match_token(&[Star, Slash]) {
            let _ = self.factor();
            return Err(ParseError {
                token: self.previous().clone(),
                msg: "Missing left-hand operand.".to_string(),
            });
        }

        Err(ParseError {
            token: self.peek().clone(),
            msg: "Expect expression.".to_string(),
        })
    }

    fn consume(&mut self, t_type: TokenType, err_msg: &str) -> Result<Token, ParseError> {
        if self.check(t_type) {
            return Ok(self.advance().clone());
        }

        Err(ParseError {
            token: self.peek().clone(),
            msg: err_msg.to_string(),
        })
    }

    // will not be used for the time being (per the book)
    // used for error recovery
    fn synchronize(&mut self) {
        use TokenType::*;
        self.advance();

        while !self.is_at_end() {
            if self.previous().t_type == TokenType::Semicolon {
                return;
            }

            if let Class | Fun | Var | For | If | While | Print | Return = self.peek().t_type {
                return;
            }
        }

        self.advance();
    }

    fn left_association_binary(
        &mut self,
        types: &[TokenType],
        expr_fn: fn(&mut Self) -> Result<Expr, ParseError>,
    ) -> Result<Expr, ParseError> {
        let mut expr = expr_fn(self)?;
        while self.match_token(types) {
            let op = self.previous().clone();
            let right = expr_fn(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t_type in types {
            if self.check(*t_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, t_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().t_type == t_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().t_type, TokenType::EOF)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
