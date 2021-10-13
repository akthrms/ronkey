use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

type ParseError = String;

/// 優先順位
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

        let expression = self.parse_expression(&Precedence::Lowest)?;
        let statement = Statement::LetStatement {
            name,
            value: expression,
        };

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.next_token();

        let expression = self.parse_expression(&Precedence::Lowest)?;
        let statement = Statement::ReturnStatement(expression);

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expression = self.parse_expression(&Precedence::Lowest)?;
        let statement = Statement::ExpressionStatement(expression);

        while self.is_peek_token(&Token::Semicolon) {
            self.next_token();
        }

        Ok(statement)
    }

    fn parse_expression(&mut self, precedence: &Precedence) -> Result<Expression, ParseError> {
        match &self.current_token {
            Token::Ident(value) => Ok(Expression::Identifier(value.clone())),
            Token::Int(value) => Ok(Expression::Integer(value.clone())),
            _ => unimplemented!(),
        }
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

    let tests = [("x", 5), ("y", 10), ("foobar", 838383)];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(
            statement,
            &Statement::LetStatement {
                name: test.0.to_string(),
                value: Expression::Integer(test.1)
            }
        );
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

    let tests = [5, 10, 993322];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(
            statement,
            &Statement::ReturnStatement(Expression::Integer(test))
        );
    }
}

#[test]
fn test_identifier_statements() {
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
        Statement::ExpressionStatement(Expression::Identifier("foobar".to_string()))
    );
}

#[test]
fn test_integer_statements() {
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
        Statement::ExpressionStatement(Expression::Integer(5))
    );
}
