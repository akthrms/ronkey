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
    /// デフォルト
    Default,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Null => write!(f, "null"),
            Self::Default => write!(f, ""),
        }
    }
}
