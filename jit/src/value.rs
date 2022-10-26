use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait Class: std::fmt::Debug {
    fn set_attribute(&mut self, name: &str, val: Arc<Value>);

    fn get_attribute(&self, name: &str) -> Arc<Value>;

    fn set_index(&mut self, idx: u32, val: Arc<Value>);

    fn get_index(&self, idx: u32) -> Arc<Value>;
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
    Array(Vec<Arc<Value>>),
    Object(HashMap<String, Arc<Value>>),
    Function(u64),
    Method{class: Arc<Mutex<dyn Class>>, func: u64},
    Class(Arc<Mutex<dyn Class>>),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Number(a) => {
                if let Self::Number(b) = other {
                    return *a == *b;
                }
            }
            Self::Boolean(a) => {
                if let Self::Boolean(b) = other {
                    return *a == *b;
                }
            }
            Self::Str(a) => {
                if let Self::Str(b) = other {
                    return *a == *b
                }
            }
            Self::Array(a) => {
                if let Self::Array(b) = other {
                    return *a == *b;
                }
            }
            Self::Object(a) => {
                if let Self::Object(b) = other {
                    return *a == *b;
                }
            }
            Self::Function(a) => {
                if let Self::Function(b) = other {
                    return *a == *b;
                }
            }
            Self::Method { class, func } => {
                let a = func;
                if let Self::Method { class, func  } = other {
                    return *a == *func
                }
            }
            Self::Class(a) => {
                if let Self::Class(b) = other {
                    return false;
                }
            }
            Self::Null => {
                if let Self::Null = other {
                    return true;
                }
            }
        }

        false
    }

    fn ne(&self, other: &Self) -> bool {
       !self.eq(other) 
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        #[cfg(feature = "trace")]
        log::trace!(target: "typescript.value", "value dropped: {:?}", self);
    }
}
