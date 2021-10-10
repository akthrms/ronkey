use crate::token::*;
use std::iter::FromIterator;

struct Lexer {
    input: Vec<char>,
    /// 入力における現在の位置（現在の文字を指し示す）
    position: usize,
    /// これから読み込む位置（現在の文字の次）
    read_position: usize,
    /// 現在検査中の文字
    ch: char,
}

impl Lexer {
    fn new(input: &str) -> Self {
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

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '=' => Token::Assign,
            ';' => Token::Semicolon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            '+' => Token::Plus,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '\u{0}' => Token::Eof,
            _ => {
                if self.is_letter() {
                    return self.read_ident();
                } else if self.is_digit() {
                    return self.read_int();
                } else {
                    Token::Illegal
                }
            }
        };

        self.read_char();
        token
    }

    fn read_ident(&mut self) -> Token {
        let start_position = self.position;

        while self.is_letter() {
            self.read_char();
        }

        let ident = String::from_iter(&self.input[start_position..self.position]);

        match ident.as_str() {
            "fn" => Token::Function,
            "let" => Token::Let,
            _ => Token::Ident(ident),
        }
    }

    fn read_int(&mut self) -> Token {
        let start_position = self.position;

        while self.is_digit() {
            self.read_char();
        }

        let int = String::from_iter(&self.input[start_position..self.position]);

        match int.parse() {
            Ok(i) => Token::Int(i),
            Err(_) => Token::Illegal,
        }
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

#[test]
fn test_next_token() {
    let input = r"
let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
";

    let tests = [
        Token::Let,
        Token::Ident("five".to_string()),
        Token::Assign,
        Token::Int(5),
        Token::Semicolon,
        Token::Let,
        Token::Ident("ten".to_string()),
        Token::Assign,
        Token::Int(10),
        Token::Semicolon,
        Token::Let,
        Token::Ident("add".to_string()),
        Token::Assign,
        Token::Function,
        Token::LParen,
        Token::Ident("x".to_string()),
        Token::Comma,
        Token::Ident("y".to_string()),
        Token::RParen,
        Token::LBrace,
        Token::Ident("x".to_string()),
        Token::Plus,
        Token::Ident("y".to_string()),
        Token::Semicolon,
        Token::RBrace,
        Token::Semicolon,
        Token::Let,
        Token::Ident("result".to_string()),
        Token::Assign,
        Token::Ident("add".to_string()),
        Token::LParen,
        Token::Ident("five".to_string()),
        Token::Comma,
        Token::Ident("ten".to_string()),
        Token::RParen,
        Token::Semicolon,
        Token::Eof,
    ];

    let mut lexer = Lexer::new(input);

    for test in tests.iter() {
        let token = lexer.next_token();
        assert_eq!(token, *test);
    }
}