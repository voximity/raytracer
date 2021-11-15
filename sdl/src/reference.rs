use crate::interpret::Value;

/// A reference object value.
#[derive(Debug, Clone)]
pub enum RefObject {
    Array(Vec<Value>),
}

impl From<RefObject> for Value {
    fn from(other: RefObject) -> Self {
        match other {
            RefObject::Array(v) => Self::Array(v),
        }
    }
}
