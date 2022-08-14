use crate::parser::{Expr, Value};
use crate::scanner::{TokenInfo, TokenType};
#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn evaluate(expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => Interpreter::evaluate_binary(left.as_ref(), operator, right.as_ref()),
            Expr::Unary { operator, right } => {
                Interpreter::evaluate_unary(operator, right.as_ref())
            }
            Expr::Grouping(e) => Interpreter::evaluate(e),
            Expr::Literal(v) => Ok(v.clone()),
        }
    }

    fn evaluate_unary(operator: &TokenInfo, right: &Expr) -> Result<Value, String> {
        let right = Interpreter::evaluate(right)?;
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
    fn evaluate_binary(left: &Expr, operator: &TokenInfo, right: &Expr) -> Result<Value, String> {
        let left = Interpreter::evaluate(left)?;
        let right = Interpreter::evaluate(right)?;
        match operator.token_type {
            TokenType::Plus => Interpreter::add_values(left, right),
            TokenType::Minus => Interpreter::subtract_values(left, right),
            TokenType::Star => Interpreter::multiply_values(left,right),
            TokenType::Slash => Interpreter::divide_values(left,right),

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
            (Value::Number(left), Value::Number(right)) => {
                Ok(Value::Number(left - right))
            }
            (_, _) => Err("To subtract operands must be two numbers".to_string()),
        }
    }
}
