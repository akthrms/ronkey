use crate::token::Token;
use std::fmt;

/// 文
#[derive(Debug, PartialEq)]
pub enum Statement {
    /// let
    Let { name: String, value: Expression },
    /// return
    Return(Expression),
    /// 式
    Expression(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let { name, value } => write!(f, "let {} = {};", name, value),
            Self::Return(expression) => write!(f, "return {};", expression),
            Self::Expression(expression) => write!(f, "{}", expression),
        }
    }
}

/// 式
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// 文字列
    Identifier(String),
    /// 数値
    Integer(isize),
    /// 前置演算子
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(value) => write!(f, "{}", value),
            Expression::Integer(value) => write!(f, "{}", value),
            Expression::Prefix { operator, right } => write!(f, "({}{})", operator, right),
        }
    }
}

/// プログラム
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self { statements: vec![] }
    }
}
