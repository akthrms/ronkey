use std::fmt;

/// オブジェクト
#[derive(Debug, PartialEq)]
pub enum Object {
    /// 整数
    Integer(isize),
    /// 真偽値
    Boolean(bool),
    /// null
    Null,
    /// return文
    Return(Box<Object>),
    /// デフォルト
    Default,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Null => write!(f, "null"),
            Self::Return(object) => write!(f, "{}", object),
            Self::Default => write!(f, ""),
        }
    }
}

impl From<Object> for String {
    fn from(object: Object) -> Self {
        match object {
            Object::Integer(_) => "Integer".to_string(),
            Object::Boolean(_) => "Boolean".to_string(),
            Object::Null => "Null".to_string(),
            _ => "".to_string(),
        }
    }
}
