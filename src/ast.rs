/// 文
pub enum Statement {
    /// let
    LetStatement { name: String, value: Expression },
}

/// 式
pub enum Expression {
    /// 文字列
    Identifier { value: String },
}

pub struct Program {
    pub statements: Vec<Statement>,
}
