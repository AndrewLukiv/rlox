use crate::scanner::{TokenInfo, TokenType};
use crate::util::format_number;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

#[derive(Debug,Clone)]
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
    Variable(TokenInfo),

    Assign {
        name: TokenInfo,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: TokenInfo,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: TokenInfo,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(str) => write!(f, "{str}"),
            Value::Number(n) => write!(f, "{}", format_number(n)),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(str) => write!(f, "{str:?}"),
            Value::Number(n) => write!(f, "{}", format_number(n)),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
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
            Expr::Literal(value) => write!(f, "{value:?}"),
            Expr::Variable(name_token) => write!(f, "{}", name_token.lexeme),
            Expr::Assign { name, value } => {
                parenthesize(f, format!("assign {} to", name.lexeme), &[value.as_ref()])
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => parenthesize(f, operator.lexeme.clone(), &[left.as_ref(), right.as_ref()]),
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

    fn get_matched_token(&mut self, token_types: &[TokenType]) -> Option<TokenInfo> {
        for t in token_types {
            if self.check(t) {
                self.advance();
                return Some(self.previous().clone());
            }
        }
        None
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

    fn new_error(
        &self,
        error_type: ParsingErrorType,
        message: impl Display,
        expression: Option<Expr>,
    ) -> ParsingError {
        self.new_error_on_line(error_type, message, self.previous().line, expression)
    }

    fn new_error_on_line(
        &self,
        error_type: ParsingErrorType,
        message: impl Display,
        line: usize,
        expression: Option<Expr>,
    ) -> ParsingError {
        ParsingError {
            error_type,
            message: message.to_string(),
            line,
            expression,
        }
    }
    fn new_expr_stmt_error(&self, message: impl Display, expr: Expr) -> ParsingError {
        self.new_error(ParsingErrorType::Stmt, message, Some(expr))
    }
    fn new_stmt_error(&self, message: impl Display) -> ParsingError {
        self.new_error(ParsingErrorType::Stmt, message, None)
    }
    fn new_expr_error(&self, message: impl Display) -> ParsingError {
        self.new_error(ParsingErrorType::Expr, message, None)
    }
    fn new_expr_error_on_line(&self, message: impl Display, line: usize) -> ParsingError {
        self.new_error_on_line(ParsingErrorType::Expr, message, line, None)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParsingError>> {
        let mut statments: Vec<Stmt> = Vec::new();
        let mut errors: Vec<ParsingError> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(declaration) => statments.push(declaration),
                Err(e) => errors.extend(e),
            }
        }
        return if errors.len() == 0 {
            Ok(statments)
        } else {
            Err(errors)
        };
    }

    pub fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.assigment()
    }

    fn assigment(&mut self) -> Result<Expr, ParsingError> {
        let expr = self.or()?;

        if self.match_tokens(&[TokenType::Equal]) {
            let equals_token = self.previous().clone();
            let value = self.assigment()?;

            return match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => {
                    Err(self.new_expr_error_on_line("Invalid assigment target", equals_token.line))
                }
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParsingError> {
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

    fn comparison(&mut self) -> Result<Expr, ParsingError> {
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

    fn term(&mut self) -> Result<Expr, ParsingError> {
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
    fn factor(&mut self) -> Result<Expr, ParsingError> {
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

    fn unary(&mut self) -> Result<Expr, ParsingError> {
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

    fn primary(&mut self) -> Result<Expr, ParsingError> {
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
        if self.match_tokens(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        self.match_tokens(&[TokenType::LeftParen]);

        let expr = self.expression()?;
        if !self.match_tokens(&[TokenType::RightParen]) {
            return Err(self.new_expr_error("Unterminated parenthesize"));
        }
        return Ok(Expr::Grouping(Box::new(expr)));
    }

    fn statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        if self.match_tokens(&[TokenType::For]) {
            return self.for_statment();
        }
        if self.match_tokens(&[TokenType::While]) {
            return self.while_statment();
        }
        if self.match_tokens(&[TokenType::Print]) {
            return self.print_statment();
        }
        if self.match_tokens(&[TokenType::LeftBrace]) {
            return self.block_statment();
        }
        if self.match_tokens(&[TokenType::If]) {
            return self.if_statment();
        }
        self.expression_statment()
    }
    fn print_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let expr = self.expression().map_err(|e| vec![e])?;
        if !self.match_tokens(&[TokenType::Semicolon]) {
            return Err(vec![self.new_stmt_error("Expect ';' after value")]);
        }
        Ok(Stmt::Print(expr))
    }
    fn expression_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let expr = self.expression().map_err(|e| vec![e])?;
        if !self.match_tokens(&[TokenType::Semicolon]) {
            return Err(vec![
                self.new_expr_stmt_error("Expect ';' after expression", expr)
            ]);
        }
        Ok(Stmt::Expression(expr))
    }

    fn declaration(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        if self.match_tokens(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statment()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let name = self
            .get_matched_token(&[TokenType::Identifier])
            .ok_or_else(|| vec![self.new_stmt_error("Expect variable name.")])?;
        let mut initializer: Option<Expr> = None;
        if self.match_tokens(&[TokenType::Equal]) {
            initializer = Some(self.expression().map_err(|e| vec![e])?)
        }
        if !self.match_tokens(&[TokenType::Semicolon]) {
            return Err(vec![
                self.new_stmt_error("Expect ';' after variable declaration.")
            ]);
        }
        Ok(Stmt::Var { name, initializer })
    }

    fn block_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let mut statments = Vec::new();
        let mut errors = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(s) => statments.push(s),
                Err(e) => errors.extend(e),
            }
        }
        if !self.match_tokens(&[TokenType::RightBrace]) {
            errors.push(self.new_stmt_error("Expect '}' after block"))
        };
        if errors.len() == 0 {
            Ok(Stmt::Block(statments))
        } else {
            Err(errors)
        }
    }

    fn if_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let mut errors = Vec::new();
        if !self.match_tokens(&[TokenType::LeftParen]) {
            errors.push(self.new_stmt_error("Expect '(' after if ."));
            return Err(errors);
        }
        let condition_parse_result = self.expression();
        let mut condition = None;
        match condition_parse_result {
            Err(e) => errors.push(e),
            Ok(expr) => condition = Some(expr),
        }
        if !self.match_tokens(&[TokenType::RightParen]) {
            errors.push(self.new_stmt_error("Expect ')' after if condition."));
        }
        let then_branch_parse_result = self.statment();
        let mut then_branch = None;
        match then_branch_parse_result {
            Err(e) => errors.extend(e),
            Ok(stmt) => then_branch = Some(stmt),
        }
        let mut else_branch = None;
        if self.match_tokens(&[TokenType::Else]) {
            match self.statment() {
                Ok(stmt) => else_branch = Some(Box::new(stmt)),
                Err(e) => errors.extend(e),
            }
        };
        if errors.len() == 0 {
            Ok(Stmt::If {
                condition: condition.unwrap(),
                then_branch: Box::new(then_branch.unwrap()),
                else_branch,
            })
        } else {
            Err(errors)
        }
    }

    fn or(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.and()?;
        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.equality()?;
        while self.match_tokens(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn while_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let mut errors = Vec::new();
        if !self.match_tokens(&[TokenType::LeftParen]) {
            errors.push(self.new_stmt_error("Expect '(' after 'while'."));
            return Err(errors);
        }
        let condition_parse_result = self.expression();
        let mut condition = None;
        match condition_parse_result {
            Err(e) => errors.push(e),
            Ok(expr) => condition = Some(expr),
        }
        if !self.match_tokens(&[TokenType::RightParen]) {
            errors.push(self.new_stmt_error("Expect ')' after condition."));
        }
        let body_parse_result = self.statment();
        let mut body = None;
        match body_parse_result {
            Err(e) => errors.extend(e),
            Ok(stmt) => body = Some(stmt),
        };
        if errors.len() == 0 {
            Ok(Stmt::While {
                condition: condition.unwrap(),
                body: Box::new(body.unwrap()),
            })
        } else {
            Err(errors)
        }
    }

    fn for_statment(&mut self) -> Result<Stmt, Vec<ParsingError>> {
        let mut errors = Vec::new();
        if !self.match_tokens(&[TokenType::LeftParen]) {
            errors.push(self.new_stmt_error("Expect '(' after 'for'."));
            return Err(errors);
        };
        let mut initializer = None;
        if self.match_tokens(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else if !self.match_tokens(&[TokenType::Semicolon]) {
            initializer = Some(self.expression_statment()?);
        };

        let mut condition = None;
        if !self.check(&TokenType::Semicolon) {
            match self.expression() {
                Ok(expr) => condition = Some(expr),
                Err(e) => errors.push(e),
            }
        };
        if !self.match_tokens(&[TokenType::Semicolon]) {
            errors.push(self.new_stmt_error("Expect ';' after loop condition."));
            return Err(errors);
        };
        let mut increment = None;
        if !self.check(&TokenType::RightParen) {
            match self.expression() {
                Ok(expr) => increment = Some(expr),
                Err(e) => errors.push(e),
            }
        };
        if !self.match_tokens(&[TokenType::RightParen]) {
            errors.push(self.new_stmt_error("Expect ')' after for clauses."));
        }
        let mut body = self.statment().or_else(|e| {
            errors.extend(e);
            Err(errors.clone())
        })?;
        if errors.len() == 0 {
            if let Some(increment) = increment {
                body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
            }

            let condition = condition.unwrap_or_else(|| Expr::Literal(Value::Boolean(true)));
            body = Stmt::While {
                condition,
                body: Box::new(body),
            };
            if let Some(initializer) = initializer {
               body=Stmt::Block(vec![initializer,body]);
            };
            Ok(body)
        } else {
            Err(errors)
        }
    }
}

#[derive(Clone,Debug, PartialEq, Eq)]
pub enum ParsingErrorType {
    Expr,
    Stmt,
}

impl Display for ParsingErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingErrorType::Expr => write!(f, "expression"),
            ParsingErrorType::Stmt => write!(f, "statment"),
        }
    }
}
#[derive(Debug,Clone)]
pub struct ParsingError {
    pub error_type: ParsingErrorType,
    pub message: String,
    pub line: usize,
    pub expression: Option<Expr>,
}
