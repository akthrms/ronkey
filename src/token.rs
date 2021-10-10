#[derive(Debug, PartialEq)]
pub enum Token {
    /// 不正な文字
    Illegal(char),
    /// 終端
    Eof,

    // 識別子 + リテラル
    /// 文字列
    Ident(String),
    /// 数値
    Int(isize),

    // 演算子
    /// =
    Assign,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Asterisk,
    /// /
    Slash,
    /// !
    Bang,

    /// <
    Lt,
    /// >
    Gt,
    /// ==
    Eq,
    /// !=
    Ne,

    // デリミタ
    /// ,
    Comma,
    /// ;
    Semicolon,

    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,

    // キーワード
    /// fn
    Function,
    /// let
    Let,
    /// true
    True,
    /// false
    False,
    /// if
    If,
    /// else
    Else,
    /// return
    Return,
}
