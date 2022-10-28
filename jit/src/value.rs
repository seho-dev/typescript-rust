use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait Class: std::fmt::Debug {
    fn set(&mut self, name: Arc<Value>, val: Arc<Value>);

    fn get(&self, name: Arc<Value>) -> Arc<Value>;

    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
    Array(Vec<Arc<Value>>),
    Object(HashMap<String, Arc<Value>>),
    Function(u64),
    Method {
        class: Arc<Mutex<dyn Class>>,
        func: u64,
    },
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

    pub fn set(&mut self, _name: Arc<Value>, _val: Arc<Value>) {}

    pub fn get(&self, name: Arc<Value>) -> Arc<Value> {
        match self {
            Self::Object(a) => {
                if let Value::Str(name) = &*name {
                    if let Some(val) = a.get(name) {
                        return val.clone();
                    }
                }
            }
            Self::Class(c) => {
                let clss = c.lock().unwrap();
                return clss.get(name);
            }
            _ => {}
        }

        Arc::new(Value::Null)
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
                    return *a == *b;
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
            Self::Method { class: _, func } => {
                let a = func;
                if let Self::Method { class: _, func } = other {
                    return *a == *func;
                }
            }
            Self::Class(_a) => {
                if let Self::Class(_b) = other {
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
