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
}
