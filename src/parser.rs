use crate::ast::{Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    fn new(lexer: &'a mut Lexer) -> Self {
        let mut parser = Parser {
            lexer: lexer,
            current_token: Token::Eof,
            peek_token: Token::Eof,
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
        unimplemented!();
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

    assert_eq!(
        program.statements.len(),
        3,
        "statements does not contain 3 statements. got={}",
        program.statements.len()
    );

    let tests = ["x", "y", "foobar"];

    for (i, test) in tests.iter().enumerate() {
        let statement = &program.statements[i];
        match statement {
            Statement::LetStatement { name, value: _ } => {
                assert_eq!(name, test, "name not {}. got={}", test, name);
            }
        }
    }
}
