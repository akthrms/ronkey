use crate::ast::{Expression, Statement};
use crate::evaluator::Environment;
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
    /// let文
    Let,
    /// デフォルト
    Default,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{} : Integer", value),
            Self::Boolean(value) => write!(f, "{} : Boolean", value),
            Self::Strings(value) => write!(f, "{} : String", value),
            Self::Null => write!(f, "Null"),
            Self::Return(object) => write!(f, "{} : {}", object, object.get_type()),
            Self::Function { .. } => write!(f, "Function"),
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
            Self::Null => "Null".to_string(),
            Self::Function { .. } => "Function".to_string(),
            _ => "".to_string(),
        }
    }
}
