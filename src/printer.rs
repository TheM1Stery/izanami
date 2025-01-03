use crate::{ast::Expr, token::LiteralType};

pub fn pretty_print(expr: &Expr) -> String {
    match expr {
        Expr::Binary { left, op, right } => parenthesize(&op.lexeme, &[left, right]),
        Expr::Grouping { expression } => parenthesize("group", &[expression]),
        Expr::Literal { value } => match value {
            Some(LiteralType::String(v)) => v.to_string(),
            Some(LiteralType::Number(v)) => v.to_string(),
            Some(LiteralType::Bool(v)) => v.to_string(),
            Some(LiteralType::Nil) => "Nil".to_string(),
            None => "None".to_string(),
        },
        Expr::Unary { op, right } => parenthesize(&op.lexeme, &[right]),
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut parenthesized = format!("({name}");

    for expr in exprs {
        parenthesized.push(' ');
        parenthesized.push_str(&pretty_print(expr));
    }

    parenthesized.push(')');

    parenthesized
}

#[cfg(test)]
mod test {
    use crate::token::{Token, TokenType};

    use super::*;
    use Expr::*;

    #[test]
    fn equal_print_binary() {
        use TokenType::*;
        let expression = Binary {
            left: Box::new(Literal {
                value: Some(LiteralType::Number(10.2)),
            }),
            op: Token {
                t_type: Plus,
                lexeme: "+".to_string(),
                literal: None,
                line: 0,
            },
            right: Box::new(Literal {
                value: Some(LiteralType::Number(10.2)),
            }),
        };

        let actual = pretty_print(&expression);
        let expected = "(+ 10.2 10.2)";

        assert_eq!(actual, expected);
    }

    #[test]
    // tests all cases
    fn equal_test_whole() {
        use TokenType::*;
        let expression = Binary {
            left: Box::new(Unary {
                op: Token::new(Minus, "-", None, 0),
                right: Box::new(Expr::Literal {
                    value: Some(LiteralType::number_literal(123.0)),
                }),
            }),
            op: Token::new(Star, "*", None, 0),
            right: Box::new(Grouping {
                expression: Box::new(Literal {
                    value: Some(LiteralType::number_literal(45.67)),
                }),
            }),
        };

        let actual = pretty_print(&expression);
        let expected = "(* (- 123) (group 45.67))";

        assert_eq!(expected, actual);
    }
}
