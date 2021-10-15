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

    fn parse_block_statement(&mut self) -> Result<Statement, ParseError> {
        let mut statements = vec![];

        self.next_token();

        while !self.is_current_token(&Token::RBrace) && !self.is_current_token(&Token::Eof) {
            let statement = self.parse_statement()?;
            statements.push(statement);

            self.next_token();
        }

        Ok(Statement::Block(statements))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        let mut expression = match &self.current_token {
            Token::Ident(value) => Expression::Identifier(value.clone()),
            Token::Int(value) => Expression::Integer(value.clone()),
            Token::Bang | Token::Minus => self.parse_prefix_expression()?,
            Token::True => Expression::Boolean(true),
            Token::False => Expression::Boolean(false),
            Token::LParen => self.parse_grouped_expression()?,
            Token::If => self.parse_if_expression()?,
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

    fn parse_grouped_expression(&mut self) -> Result<Expression, ParseError> {
        self.next_token();

        let grouped = self.parse_expression(Precedence::Lowest)?;
        let expression = Expression::Grouped(Box::new(grouped));

        self.expect_peek(&Token::RParen)?;

        Ok(expression)
    }

    fn parse_if_expression(&mut self) -> Result<Expression, ParseError> {
        self.expect_peek(&Token::LParen)?;
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;

        self.expect_peek(&Token::RParen)?;
        self.expect_peek(&Token::LBrace)?;

        let consequence = self.parse_block_statement()?;

        let expression = Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: if self.is_peek_token(&Token::Else) {
                self.next_token();
                self.expect_peek(&Token::LBrace)?;

                let alternative = self.parse_block_statement()?;
                Some(Box::new(alternative))
            } else {
                None
            },
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
        Statement::Let {
            name: "x".to_string(),
            value: Expression::Integer(5),
        },
        Statement::Let {
            name: "y".to_string(),
            value: Expression::Integer(10),
        },
        Statement::Let {
            name: "foobar".to_string(),
            value: Expression::Integer(838383),
        },
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &test);
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
        Statement::Return(Expression::Integer(5)),
        Statement::Return(Expression::Integer(10)),
        Statement::Return(Expression::Integer(993322)),
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &test);
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
!true;
!false;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 4);

    let tests = [
        Statement::Expression(Expression::Prefix {
            operator: Token::Bang,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Prefix {
            operator: Token::Minus,
            right: Box::new(Expression::Integer(15)),
        }),
        Statement::Expression(Expression::Prefix {
            operator: Token::Bang,
            right: Box::new(Expression::Boolean(true)),
        }),
        Statement::Expression(Expression::Prefix {
            operator: Token::Bang,
            right: Box::new(Expression::Boolean(false)),
        }),
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &test);
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
true == true;
true != false;
false == false;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 11);

    let tests = [
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Plus,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Minus,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Asterisk,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Slash,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Gt,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Lt,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Eq,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Integer(5)),
            operator: Token::Ne,
            right: Box::new(Expression::Integer(5)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Boolean(true)),
            operator: Token::Eq,
            right: Box::new(Expression::Boolean(true)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Boolean(true)),
            operator: Token::Ne,
            right: Box::new(Expression::Boolean(false)),
        }),
        Statement::Expression(Expression::Infix {
            left: Box::new(Expression::Boolean(false)),
            operator: Token::Eq,
            right: Box::new(Expression::Boolean(false)),
        }),
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &test);
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
3 + 4 * 5 == 3 * 1 + 4 * 5;
true;
false;
3 > 5 == false;
3 < 5 == true;
1 + (2 + 3) + 4;
(5 + 5) * 2;
2 / (5 + 5);
-(5 + 5);
!(true == true);
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 22);

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
        "true",
        "false",
        "((3 > 5) == false)",
        "((3 < 5) == true)",
        "((1 + (2 + 3)) + 4)",
        "((5 + 5) * 2)",
        "(2 / (5 + 5))",
        "(-(5 + 5))",
        "(!(true == true))",
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement.to_string(), test.to_string());
    }
}

#[test]
fn test_boolean_expressions() {
    let input = r"
true;
false;
let foobar = true;
let barfoo = false;
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 4);

    let tests = [
        Statement::Expression(Expression::Boolean(true)),
        Statement::Expression(Expression::Boolean(false)),
        Statement::Let {
            name: "foobar".to_string(),
            value: Expression::Boolean(true),
        },
        Statement::Let {
            name: "barfoo".to_string(),
            value: Expression::Boolean(false),
        },
    ];

    for (statement, test) in program.statements.iter().zip(tests) {
        assert_eq!(statement, &test);
    }
}

#[test]
fn test_if_expressions() {
    let input = r"
if (x < y) { x }
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 1);

    let condition = Expression::Infix {
        left: Box::new(Expression::Identifier("x".to_string())),
        operator: Token::Lt,
        right: Box::new(Expression::Identifier("y".to_string())),
    };

    let statement_x = Statement::Expression(Expression::Identifier("x".to_string()));
    let consequence = Statement::Block(vec![statement_x]);

    assert_eq!(
        program.statements[0],
        Statement::Expression(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: None
        })
    )
}

#[test]
fn test_if_else_expressions() {
    let input = r"
if (x < y) { x } else { y }
";

    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    for error in parser.errors.iter() {
        println!("{}", error);
    }

    assert_eq!(parser.errors.len(), 0);
    assert_eq!(program.statements.len(), 1);

    let condition = Expression::Infix {
        left: Box::new(Expression::Identifier("x".to_string())),
        operator: Token::Lt,
        right: Box::new(Expression::Identifier("y".to_string())),
    };

    let statement_x = Statement::Expression(Expression::Identifier("x".to_string()));
    let consequence = Statement::Block(vec![statement_x]);

    let statement_y = Statement::Expression(Expression::Identifier("y".to_string()));
    let alternative = Statement::Block(vec![statement_y]);

    assert_eq!(
        program.statements[0],
        Statement::Expression(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: Some(Box::new(alternative))
        })
    )
}
