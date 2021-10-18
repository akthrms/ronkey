use crate::ast::{Expression, Program, Statement};
use crate::object::Object;
use crate::token::Token;

type EvaluateError = String;

pub fn evaluate(program: Program) -> Result<Object, EvaluateError> {
    let mut result = Object::Default;

    for statement in program.statements.iter() {
        result = evaluate_statement(statement)?;

        if let Object::Return(result) = result {
            return Ok(*result);
        }
    }

    Ok(result)
}

fn evaluate_statement(statement: &Statement) -> Result<Object, EvaluateError> {
    let result = match statement {
        Statement::Expression(expression) => evaluate_expression(expression)?,
        Statement::Block(statements) => evaluate_block_statement(statements)?,
        Statement::Return(expression) => evaluate_return_statement(expression)?,
        _ => unimplemented!(),
    };

    Ok(result)
}

fn evaluate_block_statement(statements: &Vec<Statement>) -> Result<Object, EvaluateError> {
    let mut result = Object::Default;

    for statement in statements {
        result = evaluate_statement(statement)?;

        if let Object::Return(_) = result {
            break;
        }
    }

    Ok(result)
}

fn evaluate_return_statement(expression: &Expression) -> Result<Object, EvaluateError> {
    let result = evaluate_expression(expression)?;
    let result = Box::new(result);
    let result = Object::Return(result);

    Ok(result)
}

fn evaluate_expression(expression: &Expression) -> Result<Object, EvaluateError> {
    let result = match expression {
        Expression::Integer(value) => Object::Integer(value.clone()),
        Expression::Boolean(value) => Object::Boolean(value.clone()),
        Expression::Prefix { operator, right } => {
            let right = evaluate_expression(right)?;
            evaluate_prefix_expression(operator, right)?
        }
        Expression::Infix {
            left,
            operator,
            right,
        } => {
            let left = evaluate_expression(left)?;
            let right = evaluate_expression(right)?;
            evaluate_infix_expression(left, operator, right)?
        }
        Expression::Grouped(expression) => evaluate_expression(expression)?,
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition = evaluate_expression(condition)?;
            evaluate_if_expression(condition, consequence, alternative)?
        }
        _ => unimplemented!(),
    };

    Ok(result)
}

fn evaluate_prefix_expression(operator: &Token, right: Object) -> Result<Object, EvaluateError> {
    let result = match operator {
        Token::Bang => evaluate_bang_prefix_expression(right)?,
        Token::Minus => evaluate_minus_prefix_expression(right)?,
        _ => {
            let right = right.get_type();
            let message = format!("unknown operator: {}{}", operator, right);
            return Err(message);
        }
    };

    Ok(result)
}

fn evaluate_bang_prefix_expression(right: Object) -> Result<Object, EvaluateError> {
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

fn evaluate_minus_prefix_expression(right: Object) -> Result<Object, EvaluateError> {
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
    left: Object,
    operator: &Token,
    right: Object,
) -> Result<Object, EvaluateError> {
    let result = match (&left, &right) {
        (Object::Integer(left), Object::Integer(right)) => {
            evaluate_integer_infix_expression(*left, operator, *right)?
        }
        (Object::Boolean(left), Object::Boolean(right)) => {
            evaluate_boolean_infix_expression(*left, operator, *right)?
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
    left: isize,
    operator: &Token,
    right: isize,
) -> Result<Object, EvaluateError> {
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
    left: bool,
    operator: &Token,
    right: bool,
) -> Result<Object, EvaluateError> {
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
    condition: Object,
    consequence: &Statement,
    alternative: &Option<Box<Statement>>,
) -> Result<Object, EvaluateError> {
    let truthy = is_truthy(condition);

    let result = match (truthy, alternative) {
        (true, _) => evaluate_statement(consequence)?,
        (_, Some(statement)) => evaluate_statement(statement)?,
        (_, _) => Object::Null,
    };

    Ok(result)
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
    use crate::evaluator::{evaluate, EvaluateError};
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    fn test_evaluate(input: &str) -> Result<Object, EvaluateError> {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        evaluate(program)
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
}
