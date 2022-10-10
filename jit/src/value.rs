use std::{collections::HashMap, sync::Arc};

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
    Array(Vec<Arc<Value>>),
    Object(HashMap<String, Arc<Value>>),
    Function(u64),
    Null,
}

impl Value {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Number(n) => *n != 0.0,
            Self::Boolean(b) => *b,
            Self::Null => false,
            _ => true,
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        #[cfg(feature = "trace")]
        log::trace!(target: "typescript.value", "value dropped: {:?}", self);
    }
}
