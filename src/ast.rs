pub enum Statement {
    LetStatement { name: String, value: Expression },
}

pub enum Expression {
    Identifier { value: String },
}

pub struct Program {
    pub statements: Vec<Statement>,
}
