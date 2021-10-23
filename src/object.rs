use crate::ast::{Expression, Statement};
use crate::evaluator::{Environment, EvalResult};
use std::fmt;

/// オブジェクト
#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    /// 整数
    Integer(isize),
    /// 真偽値
    Boolean(bool),
    /// 文字列
    Strings(String),
    /// null
    Null,
    /// return文
    Return(Box<Object>),
    /// 関数
    Function {
        parameters: Vec<Expression>,
        body: Statement,
        env: Environment,
    },
    /// 組み込み関数
    Buildin {
        function: fn(Vec<Object>) -> EvalResult,
    },
    /// 配列
    Array(Vec<Object>),
    /// let文
    Let,
    /// デフォルト
    Default,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Strings(value) => write!(f, "{}", value),
            Self::Null => write!(f, "null"),
            Self::Return(object) => write!(f, "{}", object),
            Self::Array(elements) => {
                let elements = elements
                    .iter()
                    .map(Self::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{}]", elements)
            }
            _ => write!(f, ""),
        }
    }
}

impl Object {
    pub fn get_type(&self) -> String {
        match self {
            Self::Integer(_) => "Integer".to_string(),
            Self::Boolean(_) => "Boolean".to_string(),
            Self::Strings(_) => "String".to_string(),
            Self::Null => "null".to_string(),
            Self::Function { .. } => "Function".to_string(),
            Self::Buildin { .. } => "Buildin Function".to_string(),
            Self::Array(_) => "Array".to_string(),
            _ => "".to_string(),
        }
    }
}
