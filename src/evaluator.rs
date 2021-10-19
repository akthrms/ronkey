use crate::ast::{Expression, Program, Statement};
use crate::object::Object;
use crate::token::Token;
use std::collections::HashMap;

type EvaluateError = String;
type EvaluateResult = Result<Object, EvaluateError>;

pub struct Environment {
    store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    fn get(&self, name: &String) -> EvaluateResult {
        self.store
            .get(name)
            .map(|object| object.clone())
            .ok_or(format!("identifier not found: {}", name).to_string())
    }

    fn set(&mut self, name: String, object: Object) -> EvaluateResult {
        self.store
            .insert(name, object)
            .ok_or("unexpected error occurred".to_string())
    }

    pub fn evaluate(&mut self, program: Program) -> EvaluateResult {
        let mut result = Object::Default;

        for statement in program.statements.iter() {
            result = self.evaluate_statement(statement)?;

            if let Object::Return(result) = result {
                return Ok(*result);
            }
        }

        Ok(result)
    }

    fn evaluate_statement(&mut self, statement: &Statement) -> EvaluateResult {
        let result = match statement {
            Statement::Expression(expression) => self.evaluate_expression(expression)?,
            Statement::Block(statements) => self.evaluate_block_statement(statements)?,
            Statement::Return(expression) => self.evaluate_return_statement(expression)?,
            Statement::Let { name, value } => self.evaluate_let_statement(name, value)?,
        };

        Ok(result)
    }

    fn evaluate_block_statement(&mut self, statements: &Vec<Statement>) -> EvaluateResult {
        let mut result = Object::Default;

        for statement in statements {
            result = self.evaluate_statement(statement)?;

            if let Object::Return(_) = result {
                break;
            }
        }

        Ok(result)
    }

    fn evaluate_return_statement(&mut self, expression: &Expression) -> EvaluateResult {
        let result = self.evaluate_expression(expression)?;
        let result = Box::new(result);
        let result = Object::Return(result);

        Ok(result)
    }

    fn evaluate_let_statement(&mut self, name: &Expression, object: &Expression) -> EvaluateResult {
        let result = match name {
            Expression::Identifier(name) => {
                let name = name.to_string();
                let object = self.evaluate_expression(object)?;
                self.set(name, object)?
            }
            _ => return Err("unexpected error occurred".to_string()),
        };

        Ok(result)
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> EvaluateResult {
        let result = match expression {
            Expression::Integer(value) => Object::Integer(value.clone()),
            Expression::Boolean(value) => Object::Boolean(value.clone()),
            Expression::Prefix { operator, right } => {
                let right = self.evaluate_expression(right)?;
                self.evaluate_prefix_expression(operator, right)?
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;
                self.evaluate_infix_expression(left, operator, right)?
            }
            Expression::Grouped(expression) => self.evaluate_expression(expression)?,
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let condition = self.evaluate_expression(condition)?;
                self.evaluate_if_expression(condition, consequence, alternative)?
            }
            Expression::Identifier(value) => self.evaluate_identifier_expression(value)?,
            _ => unimplemented!(),
        };

        Ok(result)
    }

    fn evaluate_prefix_expression(&mut self, operator: &Token, right: Object) -> EvaluateResult {
        let result = match operator {
            Token::Bang => self.evaluate_bang_prefix_expression(right)?,
            Token::Minus => self.evaluate_minus_prefix_expression(right)?,
            _ => {
                let right = right.get_type();
                let message = format!("unknown operator: {}{}", operator, right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_bang_prefix_expression(&mut self, right: Object) -> EvaluateResult {
        let result = match right {
            Object::Boolean(false) => Object::Boolean(true),
            Object::Null => Object::Boolean(true),
            _ => {
                let right = right.get_type();
                let message = format!("unknown operator: !{}", right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_minus_prefix_expression(&mut self, right: Object) -> EvaluateResult {
        let result = match right {
            Object::Integer(value) => Object::Integer(-value),
            _ => {
                let right = right.get_type();
                let message = format!("unknown operator: -{}", right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_infix_expression(
        &mut self,
        left: Object,
        operator: &Token,
        right: Object,
    ) -> EvaluateResult {
        let result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => {
                let left = *left;
                let right = *right;
                self.evaluate_integer_infix_expression(left, operator, right)?
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                let left = *left;
                let right = *right;
                self.evaluate_boolean_infix_expression(left, operator, right)?
            }
            _ => {
                let left = left.get_type();
                let right = right.get_type();
                let message = format!("type mismatch: {} {} {}", left, operator, right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_integer_infix_expression(
        &mut self,
        left: isize,
        operator: &Token,
        right: isize,
    ) -> EvaluateResult {
        let result = match operator {
            Token::Plus => Object::Integer(left + right),
            Token::Minus => Object::Integer(left - right),
            Token::Asterisk => Object::Integer(left * right),
            Token::Slash => Object::Integer(left / right),
            Token::Lt => Object::Boolean(left < right),
            Token::Gt => Object::Boolean(left > right),
            Token::Eq => Object::Boolean(left == right),
            Token::Ne => Object::Boolean(left != right),
            _ => {
                let message = format!("unknown operator: Integer {} Integer", operator);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_boolean_infix_expression(
        &mut self,
        left: bool,
        operator: &Token,
        right: bool,
    ) -> EvaluateResult {
        let result = match operator {
            Token::Eq => Object::Boolean(left == right),
            Token::Ne => Object::Boolean(left != right),
            _ => {
                let message = format!("unknown operator: Boolean {} Boolean", operator);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn evaluate_if_expression(
        &mut self,
        condition: Object,
        consequence: &Statement,
        alternative: &Option<Box<Statement>>,
    ) -> EvaluateResult {
        let truthy = is_truthy(condition);
        let result = match (truthy, alternative) {
            (true, _) => self.evaluate_statement(consequence)?,
            (_, Some(statement)) => self.evaluate_statement(statement)?,
            (_, _) => Object::Null,
        };

        Ok(result)
    }

    fn evaluate_identifier_expression(&mut self, name: &String) -> EvaluateResult {
        self.get(name)
    }
}

fn is_truthy(object: Object) -> bool {
    match object {
        Object::Boolean(false) => false,
        Object::Null => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluator::{Environment, EvaluateResult};
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    fn test_evaluate(input: &str) -> EvaluateResult {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let mut env = Environment::new();
        env.evaluate(program)
    }

    #[test]
    fn test_evaluate_integer() {
        let tests = [
            ("5", Object::Integer(5)),
            ("10", Object::Integer(10)),
            ("-5", Object::Integer(-5)),
            ("-10", Object::Integer(-10)),
            ("5 + 5 + 5 + 5 - 10", Object::Integer(10)),
            ("2 * 2 * 2 * 2 * 2", Object::Integer(32)),
            ("-50 + 100 + -50", Object::Integer(0)),
            ("5 * 2 + 10", Object::Integer(20)),
            ("5 + 2 * 10", Object::Integer(25)),
            ("20 + 2 * -10", Object::Integer(0)),
            ("50 / 2 * 2 + 10", Object::Integer(60)),
            ("2 * (5 + 10)", Object::Integer(30)),
            ("3 * 3 * 3 + 10", Object::Integer(37)),
            ("3 * (3 * 3) + 10", Object::Integer(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Integer(50)),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_evaluate_boolean() {
        let tests = [
            ("true", Object::Boolean(true)),
            ("false", Object::Boolean(false)),
            ("1 < 2", Object::Boolean(true)),
            ("1 > 2", Object::Boolean(false)),
            ("1 < 1", Object::Boolean(false)),
            ("1 > 1", Object::Boolean(false)),
            ("1 == 1", Object::Boolean(true)),
            ("1 != 1", Object::Boolean(false)),
            ("1 == 2", Object::Boolean(false)),
            ("1 != 2", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("false != true", Object::Boolean(true)),
            ("(1 < 2) == true", Object::Boolean(true)),
            ("(1 < 2) == false", Object::Boolean(false)),
            ("(1 > 2) == true", Object::Boolean(false)),
            ("(1 > 2) == false", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_evaluate_bang_operator() {
        let tests = [
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!5", Object::Boolean(false)),
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("!!5", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = [
            ("if (true) { 10 }", Object::Integer(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Integer(10)),
            ("if (1 < 2) { 10 }", Object::Integer(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = [
            ("return 10;", Object::Integer(10)),
            ("return 10; 9;", Object::Integer(10)),
            ("return 2 * 5; 9;", Object::Integer(10)),
            ("9; return 2 * 5; 9;", Object::Integer(10)),
            (
                "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
                Object::Integer(10),
            ),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = [
            ("5 + true;", "type mismatch: Integer + Boolean"),
            ("5 + true; 5;", "type mismatch: Integer + Boolean"),
            ("-true;", "unknown operator: -Boolean"),
            ("true + false;", "unknown operator: Boolean + Boolean"),
            ("5; true + false; 5;", "unknown operator: Boolean + Boolean"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: Boolean + Boolean",
            ),
            (
                "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
                "unknown operator: Boolean + Boolean",
            ),
        ];

        for (input, expected) in tests {
            if let Err(message) = test_evaluate(input) {
                assert_eq!(message, expected);
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = [
            ("let a = 5; a;", Object::Integer(5)),
            ("let a = 5 * 5; a;", Object::Integer(25)),
            ("let a = 5; let b = a; b;", Object::Integer(5)),
            (
                "let a = 5; let b = a; let c = a + b + 5; c;",
                Object::Integer(15),
            ),
        ];

        for (input, expected) in tests {
            if let Ok(result) = test_evaluate(input) {
                assert_eq!(result, expected);
            }
        }
    }
}
