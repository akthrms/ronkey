use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;

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
        _ => unimplemented!(),
    }
}

fn test_evaluate(input: String) -> Object {
    let mut lexer = Lexer::new(input.as_str());
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();
    evaluate(program)
}

#[test]
fn test_evaluate_integer() {
    let tests = [("5", Object::Integer(5)), ("10", Object::Integer(10))];

    for (input, expected) in tests {
        let result = test_evaluate(input.to_string());
        assert_eq!(result, expected);
    }
}
