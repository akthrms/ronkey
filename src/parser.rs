use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;
use std::collections::BTreeMap;

/// 構文解析エラー
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
    /// array[x]
    Index,
}

impl From<Token> for Precedence {
    fn from(token: Token) -> Self {
        match token {
            Token::Eq | Token::Ne => Self::Equals,
            Token::Lt | Token::Gt => Self::LessGreater,
            Token::Plus | Token::Minus => Self::Sum,
            Token::Slash | Token::Asterisk => Self::Product,
            Token::LParen => Self::Call,
            Token::LBracket => Self::Index,
            _ => Self::Lowest,
        }
    }
}

/// 構文解析器
pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
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

    pub fn exists_errors(&mut self) -> bool {
        self.errors.len() > 0
    }

    pub fn get_errors(&mut self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn parse_program(&mut self) -> Program {
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

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let name = Expression::Identifier(self.expect_peek_identifier()?);

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
            Token::Identifier(value) => Expression::Identifier(value.clone()),
            Token::Integer(value) => Expression::Integer(value.clone()),
            Token::String(value) => Expression::String(value.clone()),
            Token::Bang | Token::Minus => self.parse_prefix_expression()?,
            Token::True => Expression::Boolean(true),
            Token::False => Expression::Boolean(false),
            Token::LParen => self.parse_grouped_expression()?,
            Token::If => self.parse_if_expression()?,
            Token::Function => self.parse_function_expression()?,
            Token::LBracket => self.parse_array_expression()?,
            Token::LBrace => self.parse_map_expression()?,
            Token::Illegal(value) => {
                let message = format!("illegal char found: {}", value);
                return Err(message);
            }
            _ => {
                let message = format!("no prefix parse function for {} found", self.current_token);
                return Err(message);
            }
        };

        while !self.is_peek_token(&Token::Semicolon)
            && precedence < Precedence::from(self.peek_token.clone())
        {
            expression = match &self.peek_token {
                &Token::LParen => {
                    self.next_token();
                    self.parse_call_expression(expression)?
                }
                &Token::Plus
                | &Token::Minus
                | &Token::Asterisk
                | &Token::Slash
                | &Token::Lt
                | &Token::Gt
                | &Token::Eq
                | &Token::Ne => {
                    self.next_token();
                    self.parse_infix_expression(expression)?
                }
                &Token::LBracket => {
                    self.next_token();
                    self.parse_index_expression(expression)?
                }
                &Token::Illegal(value) => {
                    let message = format!("illegal char found: {}", value);
                    return Err(message);
                }
                _ => expression,
            };
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

    fn parse_function_expression(&mut self) -> Result<Expression, ParseError> {
        self.expect_peek(&Token::LParen)?;

        let parameters = self.parse_function_parameters()?;

        self.expect_peek(&Token::LBrace)?;

        let body = self.parse_block_statement()?;
        let expression = Expression::Function {
            parameters,
            body: Box::new(body),
        };

        Ok(expression)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut parameters = vec![];

        if self.is_peek_token(&Token::RParen) {
            self.next_token();
            return Ok(parameters);
        }

        parameters.push(Expression::Identifier(self.expect_peek_identifier()?));

        while self.is_peek_token(&Token::Comma) {
            self.next_token();
            parameters.push(Expression::Identifier(self.expect_peek_identifier()?));
        }

        self.expect_peek(&Token::RParen)?;

        Ok(parameters)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression, ParseError> {
        let arguments = self.parse_expressions(&Token::RParen)?;
        let expression = Expression::Call {
            function: Box::new(function),
            arguments,
        };

        Ok(expression)
    }

    fn parse_expressions(&mut self, token: &Token) -> Result<Vec<Expression>, ParseError> {
        let mut arguments = vec![];

        if self.is_peek_token(token) {
            self.next_token();
            return Ok(arguments);
        }

        self.next_token();

        arguments.push(self.parse_expression(Precedence::Lowest)?);

        while self.is_peek_token(&Token::Comma) {
            self.next_token();
            self.next_token();

            arguments.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek(token)?;

        Ok(arguments)
    }

    fn parse_array_expression(&mut self) -> Result<Expression, ParseError> {
        let arguments = self.parse_expressions(&Token::RBracket)?;
        let expression = Expression::Array(arguments);

        Ok(expression)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        self.next_token();

        let index = self.parse_expression(Precedence::Lowest)?;

        self.expect_peek(&Token::RBracket)?;

        let expression = Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        };

        Ok(expression)
    }

    fn parse_map_expression(&mut self) -> Result<Expression, ParseError> {
        let mut pairs = BTreeMap::new();

        while !self.is_peek_token(&Token::RBrace) {
            self.next_token();

            let key = self.parse_expression(Precedence::Lowest)?;

            self.expect_peek(&Token::Colon)?;
            self.next_token();

            let value = self.parse_expression(Precedence::Lowest)?;

            pairs.insert(key, value);

            if !self.is_peek_token(&Token::RBrace) {
                self.expect_peek(&Token::Comma)?;
            }
        }

        self.expect_peek(&Token::RBrace)?;

        let expression = Expression::Map(pairs);

        Ok(expression)
    }

    fn expect_peek_identifier(&mut self) -> Result<String, ParseError> {
        let value = match &self.peek_token {
            Token::Identifier(value) => value.to_string(),
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
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (Token::Integer(_), Token::Integer(_)) => true,
            _ => &self.current_token == token,
        }
    }

    fn is_peek_token(&mut self, token: &Token) -> bool {
        match (&self.peek_token, token) {
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (Token::Integer(_), Token::Integer(_)) => true,
            _ => &self.peek_token == token,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Expression, Statement};
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::token::Token;
    use std::collections::BTreeMap;

    fn assert_statements(tests: Vec<(&str, Statement)>) {
        for (input, expected) in tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);
            let program = parser.parse_program();

            for error in parser.errors.iter() {
                println!("{}", error);
            }

            assert!(parser.errors.len() == 0);
            assert!(program.statements.len() > 0);

            assert_eq!(program.statements[0], expected);
        }
    }

    fn assert_statements_with_string(tests: Vec<(&str, &str)>) {
        for (input, expected) in tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);
            let program = parser.parse_program();

            for error in parser.errors.iter() {
                println!("{}", error);
            }

            assert!(parser.errors.len() == 0);
            assert!(program.statements.len() > 0);

            assert_eq!(program.statements[0].to_string(), expected.to_string());
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            (
                "let x = 5;",
                Statement::Let {
                    name: Expression::Identifier("x".to_string()),
                    value: Expression::Integer(5),
                },
            ),
            (
                "let y = 10;",
                Statement::Let {
                    name: Expression::Identifier("y".to_string()),
                    value: Expression::Integer(10),
                },
            ),
            (
                "let foobar = 838383;",
                Statement::Let {
                    name: Expression::Identifier("foobar".to_string()),
                    value: Expression::Integer(838383),
                },
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 5;", Statement::Return(Expression::Integer(5))),
            ("return 10;", Statement::Return(Expression::Integer(10))),
            (
                "return 993322;",
                Statement::Return(Expression::Integer(993322)),
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_identifier_expressions() {
        let tests = vec![(
            "foobar;",
            Statement::Expression(Expression::Identifier("foobar".to_string())),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_integer_expressions() {
        let tests = vec![("5;", Statement::Expression(Expression::Integer(5)))];

        assert_statements(tests);
    }

    #[test]
    fn test_prefix_expressions() {
        let tests = vec![
            (
                "!5;",
                Statement::Expression(Expression::Prefix {
                    operator: Token::Bang,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "-15;",
                Statement::Expression(Expression::Prefix {
                    operator: Token::Minus,
                    right: Box::new(Expression::Integer(15)),
                }),
            ),
            (
                "!true;",
                Statement::Expression(Expression::Prefix {
                    operator: Token::Bang,
                    right: Box::new(Expression::Boolean(true)),
                }),
            ),
            (
                "!false;",
                Statement::Expression(Expression::Prefix {
                    operator: Token::Bang,
                    right: Box::new(Expression::Boolean(false)),
                }),
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_infix_expressions() {
        let tests = vec![
            (
                "5 + 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Plus,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 - 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Minus,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 * 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Asterisk,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 / 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Slash,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 > 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Gt,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 < 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Lt,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 == 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Eq,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "5 != 5;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    operator: Token::Ne,
                    right: Box::new(Expression::Integer(5)),
                }),
            ),
            (
                "true == true;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Boolean(true)),
                    operator: Token::Eq,
                    right: Box::new(Expression::Boolean(true)),
                }),
            ),
            (
                "true != false;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Boolean(true)),
                    operator: Token::Ne,
                    right: Box::new(Expression::Boolean(false)),
                }),
            ),
            (
                "false == false;",
                Statement::Expression(Expression::Infix {
                    left: Box::new(Expression::Boolean(false)),
                    operator: Token::Eq,
                    right: Box::new(Expression::Boolean(false)),
                }),
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("-a * b;", "((-a) * b)"),
            ("!-a;", "(!(-a))"),
            ("a + b + c;", "((a + b) + c)"),
            ("a + b - c;", "((a + b) - c)"),
            ("a * b * c;", "((a * b) * c)"),
            ("a * b / c;", "((a * b) / c)"),
            ("a + b / c;", "(a + (b / c))"),
            ("a + b * c + d / e - f;", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4;", "(3 + 4)"),
            ("-5 * 5;", "((-5) * 5)"),
            ("5 > 4 == 3 < 4;", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4;", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5;",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true;", "true"),
            ("false;", "false"),
            ("3 > 5 == false;", "((3 > 5) == false)"),
            ("3 < 5 == true;", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4;", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2;", "((5 + 5) * 2)"),
            ("2 / (5 + 5);", "(2 / (5 + 5))"),
            ("-(5 + 5);", "(-(5 + 5))"),
            ("!(true == true);", "(!(true == true))"),
            ("a + add(b * c) + d;", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8));",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f + g);",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
            (
                "a * [1, 2 ,3, 4][b * c] + d;",
                "((a * ([1, 2, 3, 4][(b * c)])) + d)",
            ),
            (
                "add(a * b[2], b[1], 2 * [1, 2][1]);",
                "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
            ),
        ];

        assert_statements_with_string(tests);
    }

    #[test]
    fn test_boolean_expressions() {
        let tests = vec![
            ("true;", Statement::Expression(Expression::Boolean(true))),
            ("false;", Statement::Expression(Expression::Boolean(false))),
            (
                "let foobar = true;",
                Statement::Let {
                    name: Expression::Identifier("foobar".to_string()),
                    value: Expression::Boolean(true),
                },
            ),
            (
                "let barfoo = false;",
                Statement::Let {
                    name: Expression::Identifier("barfoo".to_string()),
                    value: Expression::Boolean(false),
                },
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_if_expressions() {
        let tests = vec![(
            "if (x < y) { x }",
            Statement::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    operator: Token::Lt,
                    right: Box::new(Expression::Identifier("y".to_string())),
                }),
                consequence: Box::new(Statement::Block(vec![Statement::Expression(
                    Expression::Identifier("x".to_string()),
                )])),
                alternative: None,
            }),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![(
            "if (x < y) { x } else { y }",
            Statement::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    operator: Token::Lt,
                    right: Box::new(Expression::Identifier("y".to_string())),
                }),
                consequence: Box::new(Statement::Block(vec![Statement::Expression(
                    Expression::Identifier("x".to_string()),
                )])),
                alternative: Some(Box::new(Statement::Block(vec![Statement::Expression(
                    Expression::Identifier("y".to_string()),
                )]))),
            }),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_function_expressions() {
        let tests = vec![(
            "fn(x, y) { x + y; }",
            Statement::Expression(Expression::Function {
                parameters: vec![
                    Expression::Identifier("x".to_string()),
                    Expression::Identifier("y".to_string()),
                ],
                body: Box::new(Statement::Block(vec![Statement::Expression(
                    Expression::Infix {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        operator: Token::Plus,
                        right: Box::new(Expression::Identifier("y".to_string())),
                    },
                )])),
            }),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_function_parameter_parsing() {
        let tests = vec![
            (
                "fn() {}",
                Statement::Expression(Expression::Function {
                    parameters: vec![],
                    body: Box::new(Statement::Block(vec![])),
                }),
            ),
            (
                "fn(x) {}",
                Statement::Expression(Expression::Function {
                    parameters: vec![Expression::Identifier("x".to_string())],
                    body: Box::new(Statement::Block(vec![])),
                }),
            ),
            (
                "fn(x, y) {}",
                Statement::Expression(Expression::Function {
                    parameters: vec![
                        Expression::Identifier("x".to_string()),
                        Expression::Identifier("y".to_string()),
                    ],
                    body: Box::new(Statement::Block(vec![])),
                }),
            ),
        ];

        assert_statements(tests);
    }

    #[test]
    fn test_call_expressions() {
        let tests = vec![(
            "add(1, 2 * 3, 4 + 5);",
            Statement::Expression(Expression::Call {
                function: Box::new(Expression::Identifier("add".to_string())),
                arguments: vec![
                    Expression::Integer(1),
                    Expression::Infix {
                        left: Box::new(Expression::Integer(2)),
                        operator: Token::Asterisk,
                        right: Box::new(Expression::Integer(3)),
                    },
                    Expression::Infix {
                        left: Box::new(Expression::Integer(4)),
                        operator: Token::Plus,
                        right: Box::new(Expression::Integer(5)),
                    },
                ],
            }),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_string_expressions() {
        let tests = vec![(
            r#""hello world""#,
            Statement::Expression(Expression::String("hello world".to_string())),
        )];

        assert_statements(tests);
    }

    #[test]
    fn test_array_expressions() {
        let tests = vec![("[1, 2 * 2, 3 + 3]", "[1, (2 * 2), (3 + 3)]"), ("[]", "[]")];

        assert_statements_with_string(tests);
    }

    #[test]
    fn test_index_expressions() {
        let tests = vec![("myArray[1 + 1]", "(myArray[(1 + 1)])")];

        assert_statements_with_string(tests);
    }

    #[test]
    fn test_map_expressions() {
        let tests = vec![
            (r#"{"one": 1, "two": 2, "three": 3}"#, {
                let mut pairs = BTreeMap::new();

                pairs.insert(
                    Expression::String("one".to_string()),
                    Expression::Integer(1),
                );
                pairs.insert(
                    Expression::String("two".to_string()),
                    Expression::Integer(2),
                );
                pairs.insert(
                    Expression::String("three".to_string()),
                    Expression::Integer(3),
                );

                Statement::Expression(Expression::Map(pairs))
            }),
            (
                "{}",
                Statement::Expression(Expression::Map(BTreeMap::new())),
            ),
            (r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#, {
                let mut pairs = BTreeMap::new();

                pairs.insert(
                    Expression::String("one".to_string()),
                    Expression::Infix {
                        left: Box::new(Expression::Integer(0)),
                        operator: Token::Plus,
                        right: Box::new(Expression::Integer(1)),
                    },
                );
                pairs.insert(
                    Expression::String("two".to_string()),
                    Expression::Infix {
                        left: Box::new(Expression::Integer(10)),
                        operator: Token::Minus,
                        right: Box::new(Expression::Integer(8)),
                    },
                );
                pairs.insert(
                    Expression::String("three".to_string()),
                    Expression::Infix {
                        left: Box::new(Expression::Integer(15)),
                        operator: Token::Slash,
                        right: Box::new(Expression::Integer(5)),
                    },
                );

                Statement::Expression(Expression::Map(pairs))
            }),
        ];

        assert_statements(tests);
    }
}
