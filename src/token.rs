#[derive(Debug, PartialEq)]
pub enum Token {
    Illegal(char),
    Eof,

    // 識別子 + リテラル
    Ident(String),
    Int(isize),

    // 演算子
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Bang,

    Lt,
    Gt,
    Eq,
    Ne,

    // デリミタ
    Comma,
    Semicolon,

    LParen,
    RParen,
    LBrace,
    RBrace,

    // キーワード
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}
