use crate::token::Token;
use std::collections::BTreeMap;
use std::fmt;

/// 文
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Statement {
    /// let
    Let { name: Expression, value: Expression },
    /// return
    Return(Expression),
    /// 式
    Expression(Expression),
    /// ブロック
    Block(Vec<Statement>),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let { name, value } => write!(f, "let {} = {};", name, value),
            Self::Return(expression) => write!(f, "return {};", expression),
            Self::Expression(expression) => write!(f, "{}", expression),
            Self::Block(statements) => {
                for statement in statements.iter() {
                    write!(f, "{}", statement)?;
                }
                Ok(())
            }
        }
    }
}

/// 式
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Expression {
    /// 識別子
    Identifier(String),
    /// 数値
    Integer(isize),
    /// 文字列
    String(String),
    /// 前置演算子
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    /// 中置演算子
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    /// 真偽値
    Boolean(bool),
    /// グループ化
    Grouped(Box<Expression>),
    /// if
    If {
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    /// 関数
    Function {
        parameters: Vec<Expression>,
        body: Box<Statement>,
    },
    /// 呼び出し
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    /// 配列
    Array(Vec<Expression>),
    /// インデックス
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    /// マップ
    Map(BTreeMap<Expression, Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(value) => write!(f, "{}", value),
            Self::Integer(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "{}", value),
            Self::Prefix { operator, right } => write!(f, "({}{})", operator, right),
            Self::Infix {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", left, operator, right),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Grouped(expression) => write!(f, "{}", expression),
            Self::If {
                condition,
                consequence,
                alternative,
            } => match alternative {
                Some(a) => write!(f, "if {} {{ {} }} else {{ {} }}", condition, consequence, a),
                None => write!(f, "if {} {}", condition, consequence),
            },
            Self::Function { parameters, body } => {
                let parameters = parameters.iter().map(Self::to_string).collect::<Vec<_>>();
                write!(f, "fn ({}) {{ {} }}", parameters.join(", "), body)
            }
            Self::Call {
                function,
                arguments,
            } => {
                let arguments = arguments.iter().map(Self::to_string).collect::<Vec<_>>();
                write!(f, "{}({})", function, arguments.join(", "))
            }
            Self::Array(elements) => {
                let elements = elements
                    .iter()
                    .map(Self::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{}]", elements)
            }
            Self::Index { left, index } => write!(f, "({}[{}])", left, index),
            Self::Map(pairs) => {
                let pairs = pairs
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{}}}", pairs)
            }
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
