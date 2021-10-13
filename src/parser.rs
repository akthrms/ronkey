use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

type ParseError = String;

/// 優先順位
#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    /// ==
    Equals,
    /// > <
    LessGreater,
    /// +
    Sum,
    /// *
    Product,
    /// -x !x
    Prefix,
    /// myFunction(x)
    Call,
}

impl From<Token> for Precedence {
    fn from(token: Token) -> Self {
        match token {
            Token::Eq | Token::Ne => Self::Equals,
            Token::Lt | Token::Gt => Self::LessGreater,
            Token::Plus | Token::Minus => Self::Sum,
            Token::Slash | Token::Asterisk => Self::Product,
            _ => Self::Lowest,
        }
    }
}

struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn new(lexer: &'a mut Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::Eof,
            peek_token: Token::Eof,
            errors: vec![],
        };

        parser.next_token();
        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while !self.is_current_token(&Token::Eof) {
            match self.parse_statement() {
                Ok(statement) => program.statements.push(statement),
                Err(error) => self.errors.push(error),
            }

            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect_peek_ident()?;

        self.expect_peek(&Token::Assign)?;
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;
        let statement = Statement::Let { name, value };

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.next_token();

        let expression = self.parse_expression(Precedence::Lowest)?;
        let statement = Statement::Return(expression);

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        let statement = Statement::Expression(expression);

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        let mut expression = match &self.current_token {
            Token::Ident(value) => Expression::Identifier(value.clone()),
            Token::Int(value) => Expression::Integer(value.clone()),
            Token::Bang | Token::Minus => self.parse_prefix_expression()?,
            _ => {
                return Err(format!(
                    "no prefix parse function for {} found",
                    self.current_token
                ))
            }
        };

        while !self.is_peek_token(&Token::Semicolon)
            && precedence < Precedence::from(self.peek_token.clone())
        {
            self.next_token();
            expression = self.parse_infix_expression(expression)?;
        }

        Ok(expression)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, ParseError> {
        let operator = self.current_token.clone();

        self.next_token();

        let right = self.parse_expression(Precedence::Prefix)?;
        let expression = Expression::Prefix {
            operator,
            right: Box::new(right),
        };

        Ok(expression)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let operator = self.current_token.clone();
        let precedence = Precedence::from(self.current_token.clone());

        self.next_token();

        let right = self.parse_expression(precedence)?;
        let expression = Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        };

        Ok(expression)
    }

    fn expect_peek_ident(&mut self) -> Result<String, ParseError> {
        let value = match &self.peek_token {
            Token::Ident(value) => value.to_string(),
            _ => {
                return Err(format!(
                    "expected next token to be Ident, got {} instead",
                    &self.peek_token
                ))
            }
        };

        self.next_token();
        Ok(value)
    }

    fn expect_peek(&mut self, token: &Token) -> Result<(), ParseError> {
        if self.is_peek_token(token) {
            self.next_token();
            Ok(())
        } else {
            Err(format!(
                "expected next token to be {}, got {} instead",
                token, self.peek_token
            ))
        }
    }

    fn is_current_token(&mut self, token: &Token) -> bool {
        match (&self.current_token, token) {
            (Token::Ident(_), Token::Ident(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            _ => &self.current_token == token,
        }
    }

    fn is_peek_token(&mut self, token: &Token) -> bool {
        match (&self.peek_token, token) {
            (Token::Ident(_), Token::Ident(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            _ => &self.peek_token == token,
        }
    }
}

#[test]
fn test_let_statements() {
    let input = r"
let x = 5;
let y = 10;
let foobar = 838383;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 3);

    let tests = [
        ("x".to_string(), Expression::Integer(5)),
        ("y".to_string(), Expression::Integer(10)),
        ("foobar".to_string(), Expression::Integer(838383)),
    ];

    for (statement, (name, value)) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &Statement::Let { name, value });
    }
}

#[test]
fn test_return_statements() {
    let input = r"
return 5;
return 10;
return 993322;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 3);

    let tests = [
        Expression::Integer(5),
        Expression::Integer(10),
        Expression::Integer(993322),
    ];

    for (statement, expression) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &Statement::Return(expression));
    }
}

#[test]
fn test_identifier_expressions() {
    let input = r"
foobar;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 1);

    assert_eq!(
        program.statements[0],
        Statement::Expression(Expression::Identifier("foobar".to_string()))
    );
}

#[test]
fn test_integer_expressions() {
    let input = r"
5;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 1);

    assert_eq!(
        program.statements[0],
        Statement::Expression(Expression::Integer(5))
    );
}

#[test]
fn test_prefix_expressions() {
    let input = r"
!5;
-15;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 2);

    let tests = [
        (Token::Bang, Box::new(Expression::Integer(5))),
        (Token::Minus, Box::new(Expression::Integer(15))),
    ];

    for (statement, (operator, right)) in program.statements.iter().zip(tests) {
        let expression = Expression::Prefix { operator, right };
        assert_eq!(statement, &Statement::Expression(expression));
    }
}

#[test]
fn test_infix_expressions() {
    let input = r"
5 + 5;
5 - 5;
5 * 5;
5 / 5;
5 > 5;
5 < 5;
5 == 5;
5 != 5;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 8);

    let tests = [
        (
            Box::new(Expression::Integer(5)),
            Token::Plus,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Minus,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Asterisk,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Slash,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Gt,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Lt,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Eq,
            Box::new(Expression::Integer(5)),
        ),
        (
            Box::new(Expression::Integer(5)),
            Token::Ne,
            Box::new(Expression::Integer(5)),
        ),
    ];

    for (statement, (left, operator, right)) in program.statements.iter().zip(tests) {
        let expression = Expression::Infix {
            left,
            operator,
            right,
        };
        assert_eq!(statement, &Statement::Expression(expression));
    }
}

#[test]
fn test_operator_precedence_parsing() {
    let input = r"
-a * b;
!-a;
a + b + c;
a + b - c;
a * b * c;
a * b / c;
a + b / c;
a + b * c + d / e - f;
3 + 4; -5 * 5;
5 > 4 == 3 < 4;
5 < 4 != 3 > 4;
3 + 4 * 5 == 3 * 1 + 4 * 5
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 13);

    let tests = [
        "((-a) * b)",
        "(!(-a))",
        "((a + b) + c)",
        "((a + b) - c)",
        "((a * b) * c)",
        "((a * b) / c)",
        "(a + (b / c))",
        "(((a + (b * c)) + (d / e)) - f)",
        "(3 + 4)",
        "((-5) * 5)",
        "((5 > 4) == (3 < 4))",
        "((5 < 4) != (3 > 4))",
        "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement.to_string(), test.to_string());
    }
}
