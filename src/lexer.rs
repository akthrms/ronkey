use crate::token::*;
use std::iter::FromIterator;

/// 字句解析器
pub struct Lexer {
    input: Vec<char>,
    /// 入力における現在の位置（現在の文字を指し示す）
    position: usize,
    /// これから読み込む位置（現在の文字の次）
    read_position: usize,
    /// 現在検査中の文字
    ch: char,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: 0 as char,
        };

        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0 as char;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '=' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    Token::Eq
                }
                _ => Token::Assign,
            },
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '!' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    Token::Ne
                }
                _ => Token::Bang,
            },
            '<' => Token::Lt,
            '>' => Token::Gt,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '\u{0}' => Token::Eof,
            '"' => self.read_string(),
            _ => {
                if self.is_letter() {
                    return self.read_identifier();
                } else if self.is_digit() {
                    return self.read_int();
                } else {
                    Token::Illegal(self.ch)
                }
            }
        };

        self.read_char();
        token
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            0 as char
        } else {
            self.input[self.read_position]
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start_position = self.position;

        while self.is_letter() {
            self.read_char();
        }

        let identifier = String::from_iter(&self.input[start_position..self.position]);

        match identifier.as_str() {
            "fn" => Token::Function,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            _ => Token::Identifier(identifier),
        }
    }

    fn read_int(&mut self) -> Token {
        let start_position = self.position;

        while self.is_digit() {
            self.read_char();
        }

        let int = String::from_iter(&self.input[start_position..self.position]);

        match int.parse() {
            Ok(i) => Token::Integer(i),
            Err(_) => Token::Illegal(self.input[start_position]),
        }
    }

    fn read_string(&mut self) -> Token {
        let start_position = self.position + 1;

        self.read_char();

        while self.ch != '"' && self.ch != (0 as char) {
            self.read_char();
        }

        let value = String::from_iter(&self.input[start_position..self.position]);
        Token::String(value)
    }

    fn is_letter(&self) -> bool {
        self.ch.is_alphabetic()
    }

    fn is_digit(&self) -> bool {
        self.ch.is_ascii_digit()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    fn test_next_token() {
        let input = r#"
let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10;
10 != 9;
"foobar";
"foo bar";
[1, 2];
{"foo": "bar"};
"#;

        let tests = [
            Token::Let,
            Token::Identifier("five".to_string()),
            Token::Assign,
            Token::Integer(5),
            Token::Semicolon,
            Token::Let,
            Token::Identifier("ten".to_string()),
            Token::Assign,
            Token::Integer(10),
            Token::Semicolon,
            Token::Let,
            Token::Identifier("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::Semicolon,
            Token::RBrace,
            Token::Semicolon,
            Token::Let,
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::Identifier("add".to_string()),
            Token::LParen,
            Token::Identifier("five".to_string()),
            Token::Comma,
            Token::Identifier("ten".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Integer(5),
            Token::Semicolon,
            Token::Integer(5),
            Token::Lt,
            Token::Integer(10),
            Token::Gt,
            Token::Integer(5),
            Token::Semicolon,
            Token::If,
            Token::LParen,
            Token::Integer(5),
            Token::Lt,
            Token::Integer(10),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::RBrace,
            Token::Else,
            Token::LBrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::RBrace,
            Token::Integer(10),
            Token::Eq,
            Token::Integer(10),
            Token::Semicolon,
            Token::Integer(10),
            Token::Ne,
            Token::Integer(9),
            Token::Semicolon,
            Token::String("foobar".to_string()),
            Token::Semicolon,
            Token::String("foo bar".to_string()),
            Token::Semicolon,
            Token::LBracket,
            Token::Integer(1),
            Token::Comma,
            Token::Integer(2),
            Token::RBracket,
            Token::Semicolon,
            Token::LBrace,
            Token::String("foo".to_string()),
            Token::Colon,
            Token::String("bar".to_string()),
            Token::RBrace,
            Token::Semicolon,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);

        for test in tests.iter() {
            let token = lexer.next_token();
            assert_eq!(token, *test);
        }
    }
}
