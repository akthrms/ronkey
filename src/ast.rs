/// 文
pub enum Statement {
    /// let
    LetStatement { name: String, value: Expression },
}

/// 式
pub enum Expression {
    /// 文字列
    Identifier(String),
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
