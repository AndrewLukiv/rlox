use crate::scanner::{TokenInfo, TokenType};
use crate::util::format_number;
use std::fmt::Display;
#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: TokenInfo,
        right: Box<Expr>,
    },
    Unary {
        operator: TokenInfo,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Value),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, operator.lexeme.clone(), &[left.as_ref(), right.as_ref()]),
            Expr::Unary { operator, right } => {
                parenthesize(f, operator.lexeme.clone(), &[right.as_ref()])
            }
            Expr::Grouping(expr) => parenthesize(f, "group".to_string(), &[expr.as_ref()]),
            Expr::Literal(value) => match value {
                Value::String(str) => write!(f, "{:?}", str),
                Value::Number(n) => write!(f, "{}", format_number(n)),
                Value::Boolean(b) => write!(f, "{b}"),
                Value::Nil => write!(f, "nil"),
            },
        }
    }
}

fn parenthesize(
    f: &mut std::fmt::Formatter<'_>,
    name: String,
    exprs: &[&Expr],
) -> std::fmt::Result {
    write!(f, "({name}")?;
    for e in exprs.iter() {
        write!(f, " {e}")?;
    }
    write!(f, ")")
}

pub struct Parser {
    tokens: Vec<TokenInfo>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Parser {
        Parser { tokens, current: 0 }
    }
    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for t in token_types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peak().token_type == *token_type
        }
    }

    fn advance(&mut self) -> &TokenInfo {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous()
    }
    fn peak(&self) -> &TokenInfo {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &TokenInfo {
        &self.tokens[self.current - 1]
    }
    fn is_at_end(&self) -> bool {
        self.peak().token_type == TokenType::EOF
    }

    pub fn expression(&mut self) -> Result<Expr, ExprParsingError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ExprParsingError> {
        let mut expr = self.comparison()?;
        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ExprParsingError> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ExprParsingError> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, ExprParsingError> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ExprParsingError> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ExprParsingError> {
        if self.match_tokens(&[TokenType::True]) {
            return Ok(Expr::Literal(Value::Boolean(true)));
        }
        if self.match_tokens(&[TokenType::False]) {
            return Ok(Expr::Literal(Value::Boolean(false)));
        }
        if self.match_tokens(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Value::Nil));
        }
        if self.match_tokens(&[TokenType::String]) {
            return Ok(Expr::Literal(Value::String(self.previous().lexeme.clone())));
        }
        if self.match_tokens(&[TokenType::Number]) {
            return Ok(Expr::Literal(Value::Number(
                self.previous().number.unwrap(),
            )));
        }
        self.match_tokens(&[TokenType::LeftParen]);

        let expr = self.expression()?;
        if !self.match_tokens(&[TokenType::RightParen]) {
            return Err(ExprParsingError {
                message: "Unterminated parenthesize".to_string(),
                line: self.previous().line,
            });
        }
        return Ok(Expr::Grouping(Box::new(expr)));
        // }
    }
}

#[derive(Debug)]
pub struct ExprParsingError {
    message: String,
    line: usize,
}
