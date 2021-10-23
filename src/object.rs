use crate::ast::{Expression, Statement};
use crate::evaluator::{Environment, EvalResult};
use std::collections::BTreeMap;
use std::fmt;

/// オブジェクト
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Object {
    /// 整数
    Integer(isize),
    /// 真偽値
    Boolean(bool),
    /// 文字列
    String(String),
    /// null
    Null,
    /// return
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
    /// マップ
    Map(BTreeMap<MapKey, MapPair>),
    /// let
    Let,
    /// デフォルト
    Default,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "{}", value),
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
            Self::Map(pairs) => {
                let pairs = pairs
                    .iter()
                    .map(|(_, pair)| pair.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{}}}", pairs)
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
            Self::String(_) => "String".to_string(),
            Self::Null => "null".to_string(),
            Self::Function { .. } => "Function".to_string(),
            Self::Buildin { .. } => "Buildin Function".to_string(),
            Self::Array(_) => "Array".to_string(),
            _ => "".to_string(),
        }
    }
}

/// マップのキー
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MapKey {
    Integer(isize),
    Boolean(bool),
    String(String),
    Unusable,
}

impl From<&Object> for MapKey {
    fn from(object: &Object) -> Self {
        match object {
            Object::Integer(value) => MapKey::Integer(value.clone()),
            Object::Boolean(value) => MapKey::Boolean(value.clone()),
            Object::String(value) => MapKey::String(value.clone()),
            _ => MapKey::Unusable,
        }
    }
}

/// マップの値
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MapPair {
    pub key: Object,
    pub value: Object,
}

impl MapPair {
    pub fn new(key: Object, value: Object) -> Self {
        Self { key, value }
    }
}

impl fmt::Display for MapPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::object::MapKey;

    #[test]
    fn test_string_map_key() {
        let hello1 = MapKey::String("Hello World".to_string());
        let hello2 = MapKey::String("Hello World".to_string());
        let diff1 = MapKey::String("My name is johnny".to_string());
        let diff2 = MapKey::String("My name is johnny".to_string());

        assert!(hello1 == hello2);
        assert!(diff1 == diff2);
        assert!(hello1 != diff2);
    }
}
