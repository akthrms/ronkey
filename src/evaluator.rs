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
        Expression::Prefix { operator, right } => {
            let right = evaluate_expression(right);
            evaluate_prefix_expression(operator, right)
        }
        Expression::Infix {
            left,
            operator,
            right,
        } => {
            let left = evaluate_expression(left);
            let right = evaluate_expression(right);
            evaluate_infix_expression(left, operator, right)
        }
        Expression::Grouped(expression) => evaluate_expression(expression),
        _ => unimplemented!(),
    }
}

fn evaluate_prefix_expression(operator: &Token, right: Object) -> Object {
    match operator {
        Token::Bang => evaluate_bang_prefix_expression(right),
        Token::Minus => evaluate_minus_prefix_expression(right),
        _ => unimplemented!(),
    }
}

fn evaluate_bang_prefix_expression(right: Object) -> Object {
    match right {
        Object::Boolean(false) => Object::Boolean(true),
        Object::Null => Object::Boolean(true),
        _ => Object::Boolean(false),
    }
}

fn evaluate_minus_prefix_expression(right: Object) -> Object {
    match right {
        Object::Integer(value) => Object::Integer(-value),
        _ => unimplemented!(),
    }
}

fn evaluate_infix_expression(left: Object, operator: &Token, right: Object) -> Object {
    match (left, right) {
        (Object::Integer(left), Object::Integer(right)) => {
            evaluate_integer_infix_expression(left, operator, right)
        }
        (Object::Boolean(left), Object::Boolean(right)) => {
            evaluate_boolean_infix_expression(left, operator, right)
        }
        _ => unimplemented!(),
    }
}

fn evaluate_integer_infix_expression(left: isize, operator: &Token, right: isize) -> Object {
    match operator {
        Token::Plus => Object::Integer(left + right),
        Token::Minus => Object::Integer(left - right),
        Token::Asterisk => Object::Integer(left * right),
        Token::Slash => Object::Integer(left / right),
        Token::Lt => Object::Boolean(left < right),
        Token::Gt => Object::Boolean(left > right),
        Token::Eq => Object::Boolean(left == right),
        Token::Ne => Object::Boolean(left != right),
        _ => unimplemented!(),
    }
}

fn evaluate_boolean_infix_expression(left: bool, operator: &Token, right: bool) -> Object {
    match operator {
        Token::Eq => Object::Boolean(left == right),
        Token::Ne => Object::Boolean(left != right),
        _ => unimplemented!(),
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
            let result = test_evaluate(input);
            assert_eq!(result, expected);
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
