use crate::parser::{Expr, Stmt, Value};
use crate::scanner::{TokenInfo, TokenType};
use std::collections::HashMap;
use std::io::Write;
use std::iter::Rev;
use std::slice::{Iter, IterMut};

#[derive(Debug)]
struct Environment {
    scopes: Vec<VariableScope>,
}
#[derive(Debug, Default)]
struct VariableScope {
    values: HashMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            scopes: vec![VariableScope::default()],
        }
    }
    fn scopes_iter(&self) -> Rev<Iter<VariableScope>> {
        self.scopes.iter().rev()
    }
    fn scopes_iter_mut(&mut self) -> Rev<IterMut<VariableScope>> {
        self.scopes.iter_mut().rev()
    }
    fn get(&self, name: String) -> Result<&Value, String> {
        for scope in self.scopes_iter() {
            if let Some(value) = scope.values.get(&name) {
                return Ok(value);
            }
        }
        Err(format!("Undefined variable {name}."))
    }

    fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        for scope in self.scopes_iter_mut() {
            if scope.values.contains_key(&name) {
                scope.values.insert(name, value);
                return Ok(());
            }
        }
        Err(format!("Undefined variable {name}."))
    }
    fn define(&mut self, name: String, value: Value) {
        self.scopes.last_mut().unwrap().values.insert(name, value);
    }
    fn jump_in_scope(&mut self) {
        self.scopes.push(VariableScope::default())
    }
    fn jump_out_scope(&mut self) {
        if self.scopes.len() != 1 {
            self.scopes.pop();
        } else {
            panic!("Try delete global scope")
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }
    pub fn interpret(&mut self, statments: Vec<Stmt>) -> Result<(), String> {
        for stmt in statments {
            self.execute(&stmt)?;
        }
        Ok(())
    }
    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(e) => self.execute_expression(e),
            Stmt::Print(e) => self.execute_print(e),
            Stmt::Var { name, initializer } => self.execute_variable_declaration(name, initializer),
            Stmt::Block(statments) => self.execute_block(statments),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.execute_if(condition, then_branch.as_ref(), else_branch),
            Stmt::While { condition, body } => self.execute_while(condition,body.as_ref()),
        }
    }
    fn execute_block(&mut self, statments: &Vec<Stmt>) -> Result<(), String> {
        self.environment.jump_in_scope();
        for stmt in statments {
            self.execute(stmt)?
        }
        self.environment.jump_out_scope();
        Ok(())
    }
    fn execute_variable_declaration(
        &mut self,
        name: &TokenInfo,
        initializer: &Option<Expr>,
    ) -> Result<(), String> {
        let value = match initializer {
            Some(expr) => self.evaluate(&expr)?,
            None => Value::Nil,
        };
        self.environment.define(name.lexeme.clone(), value);
        Ok(())
    }
    fn execute_print(&mut self, expr: &Expr) -> Result<(), String> {
        let value = self.evaluate(expr)?;
        println!("{value}");
        std::io::stdout().flush().unwrap();
        Ok(())
    }

    fn execute_expression(&mut self, expr: &Expr) -> Result<(), String> {
        self.evaluate(expr)?;
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left.as_ref(), operator, right.as_ref()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right.as_ref()),
            Expr::Grouping(e) => self.evaluate(e),
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Variable(t) => Ok(self.environment.get(t.lexeme.clone())?.clone()),
            Expr::Assign { name, value } => self.evaluate_assigment(name, value.as_ref()),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_logical(left.as_ref(), operator, right.as_ref()),
        }
    }

    fn evaluate_assigment(&mut self, name: &TokenInfo, expr: &Expr) -> Result<Value, String> {
        let value = self.evaluate(expr)?;
        self.environment
            .assign(name.lexeme.clone(), value.clone())?;
        Ok(value)
    }
    fn evaluate_unary(&mut self, operator: &TokenInfo, right: &Expr) -> Result<Value, String> {
        let right = self.evaluate(right)?;
        match &operator.token_type {
            TokenType::Minus => {
                if let Value::Number(n) = right {
                    Ok(Value::Number(-n))
                } else {
                    Err("Operand must be number".to_string())
                }
            }
            TokenType::Bang => {
                let boolean_value = right.is_truthy();
                Ok(Value::Boolean(!boolean_value))
            }
            t => Err(format!(
                "IllegalOperation wrong operator for unary expression {:?}",
                t
            )),
        }
    }
    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &TokenInfo,
        right: &Expr,
    ) -> Result<Value, String> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match operator.token_type {
            TokenType::Plus => Interpreter::add_values(left, right),
            TokenType::Minus => Interpreter::subtract_values(left, right),
            TokenType::Star => Interpreter::multiply_values(left, right),
            TokenType::Slash => Interpreter::divide_values(left, right),

            TokenType::Less => Interpreter::compare_lt(left, right),
            TokenType::LessEqual => Interpreter::compare_le(left, right),
            TokenType::Greater => Interpreter::compare_gt(left, right),
            TokenType::GreaterEqual => Interpreter::compare_ge(left, right),

            TokenType::EqualEqual => Interpreter::is_equal(left, right),
            TokenType::BangEqual => Interpreter::is_not_equal(left, right),
            _ => todo!(),
        }
    }
    fn divide_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
            (_, _) => Err("To divide operands must be two numbers".to_string()),
        }
    }
    fn multiply_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
            (_, _) => Err("To multiply operands must be two numbers".to_string()),
        }
    }
    fn is_equal(left: Value, right: Value) -> Result<Value, String> {
        Ok(Value::Boolean(left == right))
    }
    fn is_not_equal(left: Value, right: Value) -> Result<Value, String> {
        Ok(Value::Boolean(left != right))
    }

    fn compare_lt(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
            (_, _) => Err("To compare operands must be two numbers".to_string()),
        }
    }
    fn compare_gt(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
            (_, _) => Err("To compare operands must be two numbers".to_string()),
        }
    }
    fn compare_le(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
            (_, _) => Err("To compare operands must be two numbers".to_string()),
        }
    }
    fn compare_ge(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
            (_, _) => Err("To compare operands must be two numbers".to_string()),
        }
    }
    fn add_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
            (Value::String(left), Value::String(right)) => {
                let concated_string = format!("{left}{right}");
                Ok(Value::String(concated_string))
            }
            (_, _) => Err("To add operands must be two numbers or two strings".to_string()),
        }
    }

    fn subtract_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
            (_, _) => Err("To subtract operands must be two numbers".to_string()),
        }
    }

    fn execute_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<(), String> {
        if self.evaluate(&condition)?.is_truthy() {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch.as_ref())?;
        }
        Ok(())
    }

    fn evaluate_logical(
        &mut self,
        left: &Expr,
        operator: &TokenInfo,
        right: &Expr,
    ) -> Result<Value, String> {
        let left = self.evaluate(left)?;
        match operator.token_type {
            TokenType::And if !left.is_truthy()  =>  Ok(left),
            TokenType::Or if left.is_truthy() =>  Ok(left),
            TokenType::And | TokenType::Or=>self.evaluate(right),
            _ =>  Err("For logical operation operator must be 'and' or 'or'".to_string()),
        }
    }

    fn execute_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), String> {
        while self.evaluate(condition)?.is_truthy() {
           self.execute(body)?;
        }
        Ok(())
    }
}
