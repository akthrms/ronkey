use std::fmt;

/// オブジェクト
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Object {
    /// 整数
    Integer(isize),
    /// 真偽値
    Boolean(bool),
    /// null
    Null,
    /// return文
    Return(Box<Object>),
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
            Self::Null => write!(f, "Null"),
            Self::Return(object) => write!(f, "{} : {}", object, object.get_type()),
            _ => write!(f, ""),
        }
    }
}

impl Object {
    pub fn get_type(&self) -> String {
        match self {
            Self::Integer(_) => "Integer".to_string(),
            Self::Boolean(_) => "Boolean".to_string(),
            Self::Null => "Null".to_string(),
            _ => "".to_string(),
        }
    }
}
