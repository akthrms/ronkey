use std::fmt;

/// 文
#[derive(Debug, PartialEq)]
pub enum Statement {
    /// let
    LetStatement { name: String, value: Expression },
    /// return
    ReturnStatement(Expression),
    /// 式
    ExpressionStatement(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LetStatement { name, value } => write!(f, "let {} = {};", name, value),
            Self::ReturnStatement(expression) => write!(f, "return {};", expression),
            Self::ExpressionStatement(expression) => write!(f, "{}", expression),
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
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(value) => write!(f, "{}", value),
            Expression::Integer(value) => write!(f, "{}", value),
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
