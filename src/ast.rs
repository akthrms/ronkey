enum Statement {
    LetStatement { name: Expression, value: Expression },
}

enum Expression {
    Identifier { value: String },
}

struct Program {
    statements: Vec<Statement>,
}
