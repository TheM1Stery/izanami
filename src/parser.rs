use std::fmt::Display;

use crate::{
    ast::{Expr, Stmt},
    token::{LiteralType, Token, TokenType},
    utils::{defer, expr, ScopeCall},
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    loop_depth: u32,
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
        Parser {
            tokens,
            current: 0,
            loop_depth: 0,
        }
    }

    pub fn loop_depth(&mut self) -> &mut u32 {
        &mut self.loop_depth
    }

    //pub fn parse(&mut self) -> Result<Expr, ParseError> {
    //    self.expression()
    //}

    // maps to program rule in the grammar
    pub fn parse(&mut self) -> Vec<Result<Stmt, ParseError>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let stmt = if self.match_token(&[TokenType::Fun]) {
            self.function("function")
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        stmt.inspect_err(|_| self.synchronize())
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {kind} name."),
        )?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek().clone(),
                        msg: "Can't have more than 255 parameters".to_string(),
                    });
                }

                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?;
        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token(&[TokenType::Break]) {
            return self.break_statement();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        if *self.loop_depth() == 0 {
            return Err(ParseError {
                token: self.previous().clone(),
                msg: "Must be inside a loop to use 'break'".to_string(),
            });
        }
        self.consume(TokenType::Semicolon, "Expect ';' after 'break'")?;

        Ok(Stmt::Break)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.loop_depth += 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = Box::new(self.statement()?);
        defer! {
            *self.loop_depth() += 1;
        }

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.loop_depth += 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.match_token(&[TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.match_token(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let body = match increment {
            Some(inc) => Stmt::Block {
                statements: vec![self.statement()?, Stmt::Expression { expression: inc }],
            },
            None => self.statement()?,
        };

        let condition = condition.unwrap_or(Expr::Literal {
            value: LiteralType::Bool(true),
        });

        let body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        let body = match initializer {
            Some(init) => Stmt::Block {
                statements: vec![init, body],
            },
            None => body,
        };

        defer! {
            *self.loop_depth() -= 1;
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Stmt::Print { expression })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;

        Ok(Stmt::Expression { expression })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.comma()
    }

    // Challenge #1. We're writing comma before equality, because it has the lowest precedence
    // comma -> equality ("," equality)* ;   // expression grammar
    fn comma(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        self.left_association_binary(&[Comma], Self::assignment)
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.ternary()?;

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;
            let equals = self.previous();

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(ParseError {
                token: equals.clone(),
                msg: "Invalid assignment target.".to_string(),
            });
        }

        Ok(expr)
    }

    // ternary -> equality ("?" expression : ternary)? // expression grammar
    fn ternary(&mut self) -> Result<Expr, ParseError> {
        use TokenType::*;
        let expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::OR]) {
            let op = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_token(&[TokenType::And]) {
            let op = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek().clone(),
                        msg: "Can't have more than 255 arguments".to_string(),
                    });
                }
                args.push(self.equality()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            args,
        })
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

        if self.match_token(&[Identifier]) {
            return Ok(Expr::Variable {
                name: self.previous().clone(),
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
            self.advance();
        }
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
