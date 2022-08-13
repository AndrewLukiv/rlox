use crate::scanner::Token;
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
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
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
            } => parenthesize(f, operator, &[left.as_ref(), right.as_ref()]),
            Expr::Unary { operator, right } => parenthesize(f, operator, &[right.as_ref()]),
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
    name: impl std::fmt::Display,
    exprs: &[&Expr],
) -> std::fmt::Result {
    write!(f, "({name}")?;
    for e in exprs.iter() {
        write!(f, " {e}")?;
    }
    write!(f, ")")
}
