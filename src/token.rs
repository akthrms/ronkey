use std::fmt;

#[derive(Clone, Debug, PartialEq)]
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(value) => write!(f, "{}", value),
            Token::Int(value) => write!(f, "Int({})", value),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Bang => write!(f, "!"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Eq => write!(f, "=="),
            Token::Ne => write!(f, "!="),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Function => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),
            token => write!(f, "{}", token),
        }
    }
}
