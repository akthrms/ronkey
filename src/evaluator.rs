use crate::ast::{Expression, Program, Statement};
use crate::buildin;
use crate::object::Object;
use crate::token::Token;
use std::collections::HashMap;

/// 評価エラー
pub type EvalError = String;

/// 評価結果
pub type EvalResult = Result<Object, EvalError>;

/// レスポンス
pub enum Response {
    /// 返答する
    Reply(Object),
    /// 返答しない
    NoReply,
    /// エラー
    Error(EvalError),
}

/// 環境
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Box<Environment>>,
    buildin: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
            buildin: buildin::new(),
        }
    }

    fn new_with_outer(env: Box<Environment>) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(env),
            buildin: buildin::new(),
        }
    }

    fn get(&self, name: &String) -> EvalResult {
        let result = match self.store.get(name) {
            Some(object) => object.clone(),
            None => match &self.outer {
                Some(env) => env.get(name)?,
                None => {
                    let message = format!("identifier not found: {}", name).to_string();
                    return Err(message);
                }
            },
        };

        Ok(result)
    }

    fn set(&mut self, name: String, object: Object) -> EvalResult {
        self.store.insert(name, object.clone());
        Ok(object)
    }

    pub fn eval(&mut self, program: Program) -> Response {
        let mut result = Object::Default;

        for statement in program.statements.iter() {
            result = match self.eval_statement(statement) {
                Ok(Object::Return(result)) => return Response::Reply(*result),
                Ok(result) => result,
                Err(error) => return Response::Error(error),
            }
        }

        match result {
            Object::Let => Response::NoReply,
            _ => Response::Reply(result),
        }
    }

    fn eval_statement(&mut self, statement: &Statement) -> EvalResult {
        let result = match statement {
            Statement::Expression(expression) => self.eval_expression(expression)?,
            Statement::Block(statements) => self.eval_block_statement(statements)?,
            Statement::Return(expression) => self.eval_return_statement(expression)?,
            Statement::Let { name, value } => self.eval_let_statement(name, value)?,
        };

        Ok(result)
    }

    fn eval_block_statement(&mut self, statements: &Vec<Statement>) -> EvalResult {
        let mut result = Object::Default;

        for statement in statements {
            result = self.eval_statement(statement)?;

            if let Object::Return(_) = result {
                break;
            }
        }

        Ok(result)
    }

    fn eval_return_statement(&mut self, expression: &Expression) -> EvalResult {
        let result = self.eval_expression(expression)?;
        let result = Box::new(result);
        let result = Object::Return(result);

        Ok(result)
    }

    fn eval_let_statement(&mut self, name: &Expression, object: &Expression) -> EvalResult {
        let result = match name {
            Expression::Identifier(name) => {
                let name = name.to_string();
                let object = self.eval_expression(object)?;
                self.set(name, object)?;
                Object::Let
            }
            _ => return Err("unexpected error occurred in let binding".to_string()),
        };

        Ok(result)
    }

    fn eval_expression(&mut self, expression: &Expression) -> EvalResult {
        let result = match expression {
            Expression::Integer(value) => {
                let value = *value;
                Object::Integer(value)
            }
            Expression::Boolean(value) => {
                let value = *value;
                Object::Boolean(value)
            }
            Expression::Strings(value) => {
                let value = value.to_string();
                Object::Strings(value)
            }
            Expression::Prefix { operator, right } => {
                let right = self.eval_expression(right)?;
                self.eval_prefix_expression(operator, right)?
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left = self.eval_expression(left)?;
                let right = self.eval_expression(right)?;
                self.eval_infix_expression(left, operator, right)?
            }
            Expression::Grouped(expression) => self.eval_expression(expression)?,
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let condition = self.eval_expression(condition)?;
                self.eval_if_expression(condition, consequence, alternative)?
            }
            Expression::Identifier(value) => self.eval_identifier_expression(value)?,
            Expression::Function { parameters, body } => {
                self.eval_function_expression(parameters, body)?
            }
            Expression::Call {
                function,
                arguments,
            } => {
                let function = self.eval_expression(function)?;
                let arguments = self.eval_expressions(arguments)?;
                self.apply_function(function, arguments)?
            }
        };

        Ok(result)
    }

    fn eval_prefix_expression(&mut self, operator: &Token, right: Object) -> EvalResult {
        let result = match operator {
            Token::Bang => self.eval_bang_prefix_expression(right)?,
            Token::Minus => self.eval_minus_prefix_expression(right)?,
            _ => {
                let right = right.get_type();
                let message = format!("unknown operator: {}{}", operator, right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn eval_bang_prefix_expression(&mut self, right: Object) -> EvalResult {
        let result = match right {
            Object::Boolean(false) => Object::Boolean(true),
            Object::Null => Object::Boolean(true),
            _ => Object::Boolean(false),
        };

        Ok(result)
    }

    fn eval_minus_prefix_expression(&mut self, right: Object) -> EvalResult {
        let result = match right {
            Object::Integer(value) => {
                let value = -value;
                Object::Integer(value)
            }
            _ => {
                let right = right.get_type();
                let message = format!("unknown operator: -{}", right);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn eval_infix_expression(
        &mut self,
        left: Object,
        operator: &Token,
        right: Object,
    ) -> EvalResult {
        let result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => {
                let left = *left;
                let right = *right;
                self.eval_integer_infix_expression(left, operator, right)?
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                let left = *left;
                let right = *right;
                self.eval_boolean_infix_expression(left, operator, right)?
            }
            (Object::Strings(left), Object::Strings(right)) => {
                let left = left.to_string();
                let right = right.to_string();
                self.eval_string_infix_expression(left, operator, right)?
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

    fn eval_integer_infix_expression(
        &mut self,
        left: isize,
        operator: &Token,
        right: isize,
    ) -> EvalResult {
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

    fn eval_boolean_infix_expression(
        &mut self,
        left: bool,
        operator: &Token,
        right: bool,
    ) -> EvalResult {
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

    fn eval_string_infix_expression(
        &mut self,
        left: String,
        operator: &Token,
        right: String,
    ) -> EvalResult {
        let result = match operator {
            Token::Plus => Object::Strings(format!("{}{}", left, right)),
            Token::Eq => Object::Boolean(left == right),
            Token::Ne => Object::Boolean(left != right),
            _ => {
                let message = format!("unknown operator: String {} String", operator);
                return Err(message);
            }
        };

        Ok(result)
    }

    fn eval_if_expression(
        &mut self,
        condition: Object,
        consequence: &Statement,
        alternative: &Option<Box<Statement>>,
    ) -> EvalResult {
        let result = match (is_truthy(condition), alternative) {
            (true, _) => self.eval_statement(consequence)?,
            (_, Some(statement)) => self.eval_statement(statement)?,
            (_, _) => Object::Null,
        };

        Ok(result)
    }

    fn eval_identifier_expression(&mut self, name: &String) -> EvalResult {
        let result = match (self.get(name), self.buildin.get(name)) {
            (Ok(object), _) => object,
            (Err(_), Some(object)) => object.clone(),
            (Err(error), None) => return Err(error),
        };

        Ok(result)
    }

    fn eval_function_expression(
        &mut self,
        parameters: &Vec<Expression>,
        body: &Statement,
    ) -> EvalResult {
        let result = Object::Function {
            parameters: parameters.clone(),
            body: body.clone(),
            env: self.clone(),
        };

        Ok(result)
    }

    fn eval_expressions(
        &mut self,
        expressions: &Vec<Expression>,
    ) -> Result<Vec<Object>, EvalError> {
        let mut result = vec![];

        for expression in expressions.iter() {
            result.push(self.eval_expression(expression)?);
        }

        Ok(result)
    }

    fn apply_function(&mut self, function: Object, arguments: Vec<Object>) -> EvalResult {
        let result = match &function {
            Object::Function {
                parameters,
                body,
                env,
            } => {
                self.check_arity(parameters.len(), arguments.len())?;

                let mut env = Self::new_with_outer(Box::new(env.clone()));

                for (i, parameter) in parameters.iter().enumerate() {
                    match parameter {
                        Expression::Identifier(name) => {
                            env.set(name.to_string(), arguments[i].clone())?;
                        }
                        _ => {
                            let message = format!("invalid argument index: {}", 0).to_string();
                            return Err(message);
                        }
                    }
                }

                env.eval_statement(&body)?
            }
            Object::Buildin { function } => function(arguments)?,
            _ => {
                let message = format!("not a function: {}", function.get_type()).to_string();
                return Err(message);
            }
        };

        Ok(result)
    }

    fn check_arity(&mut self, parameters: usize, arguments: usize) -> Result<(), EvalError> {
        if parameters == arguments {
            Ok(())
        } else {
            let message = format!(
                "expected arity to be {}, got {} instead",
                parameters, arguments
            )
            .to_string();
            Err(message)
        }
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
    use crate::evaluator::{Environment, Response};
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    fn test_eval(input: &str) -> Response {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let mut env = Environment::new();
        env.eval(program)
    }

    #[test]
    fn test_eval_integer() {
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
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_eval_boolean() {
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
            (r#""Hello World!" == "Hello World!""#, Object::Boolean(true)),
            (
                r#""Hello World!" != "Hello World!""#,
                Object::Boolean(false),
            ),
        ];

        for (input, expected) in tests {
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_eval_bang_operator() {
        let tests = [
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!5", Object::Boolean(false)),
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("!!5", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
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
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
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
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
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
            ("foobar", "identifier not found: foobar"),
            (r#""Hello" - "World""#, "unknown operator: String - String"),
        ];

        for (input, expected) in tests {
            match test_eval(input) {
                Response::Error(error) => assert_eq!(error, expected),
                _ => unreachable!(),
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
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_function_expressions() {
        let input = "fn(x) { x + 2; };";

        match test_eval(input) {
            Response::Reply(result) => match result {
                Object::Function {
                    parameters, body, ..
                } => {
                    assert_eq!(parameters.len(), 1);
                    assert_eq!(parameters[0].to_string(), "x".to_string());
                    assert_eq!(body.to_string(), "(x + 2)".to_string());
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_function_application() {
        let tests = [
            (
                "let identity = fn(x) { x; }; identity(5);",
                Object::Integer(5),
            ),
            (
                "let identity = fn(x) { return x; }; identity(5);",
                Object::Integer(5),
            ),
            (
                "let double = fn(x) { x * 2; }; double(5);",
                Object::Integer(10),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5, 5);",
                Object::Integer(10),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                Object::Integer(20),
            ),
            ("fn(x) { x; }(5)", Object::Integer(5)),
        ];

        for (input, expected) in tests {
            match test_eval(input) {
                Response::Reply(result) => assert_eq!(result, expected),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_closures() {
        let input = r"
let newAdder = fn(x) {
    fn(y) { x + y };
};
let addTwo = newAdder(2);
addTwo(2);
";

        let expected = Object::Integer(4);

        match test_eval(input) {
            Response::Reply(result) => assert_eq!(result, expected),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_string() {
        let input = r#""Hello World!""#;

        let expected = Object::Strings("Hello World!".to_string());

        match test_eval(input) {
            Response::Reply(result) => assert_eq!(result, expected),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;

        let expected = Object::Strings("Hello World!".to_string());

        match test_eval(input) {
            Response::Reply(result) => assert_eq!(result, expected),
            _ => unreachable!(),
        }
    }
}
