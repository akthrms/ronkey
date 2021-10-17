use crate::ast::{Expression, Program, Statement};
use crate::object::Object;
use crate::token::Token;

pub fn evaluate(program: Program) -> Object {
    let mut result = Object::Default;

    for statement in program.statements.iter() {
        result = evaluate_statement(statement)
    }

    result
}

fn evaluate_statement(statement: &Statement) -> Object {
    match statement {
        Statement::Expression(expression) => evaluate_expression(expression),
        _ => unimplemented!(),
    }
}

fn evaluate_expression(expression: &Expression) -> Object {
    match expression {
        Expression::Integer(value) => Object::Integer(value.clone()),
        Expression::Boolean(value) => Object::Boolean(value.clone()),
        Expression::Prefix { operator, right } => evaluate_prefix_expression(operator, right),
        _ => unimplemented!(),
    }
}

fn evaluate_prefix_expression(operator: &Token, right: &Expression) -> Object {
    match operator {
        Token::Bang => evaluate_bang_prefix_expression(right),
        Token::Minus => evaluate_minus_prefix_expression(right),
        _ => unreachable!(),
    }
}

fn evaluate_bang_prefix_expression(right: &Expression) -> Object {
    match evaluate_expression(right) {
        Object::Boolean(false) | Object::Null => Object::Boolean(true),
        _ => Object::Boolean(false),
    }
}

fn evaluate_minus_prefix_expression(right: &Expression) -> Object {
    match evaluate_expression(right) {
        Object::Integer(value) => Object::Integer(-value),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluator::evaluate;
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    fn test_evaluate(input: &str) -> Object {
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
        ];

        for (input, expected) in tests {
            let result = test_evaluate(input);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_evaluate_boolean() {
        let tests = [
            ("true", Object::Boolean(true)),
            ("false", Object::Boolean(false)),
        ];

        for (input, expected) in tests {
            let result = test_evaluate(input);
            assert_eq!(result, expected);
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
            let result = test_evaluate(input);
            assert_eq!(result, expected);
        }
    }
}
