enum Statement {
    LetStatement { name: Expression, value: Expression },
}

enum Expression {
    Identifier { value: String },
}

pub struct Program {
    statements: Vec<Statement>,
}
