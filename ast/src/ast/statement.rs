use std::sync::Arc;

use super::{value::Value, tstype::Type};

#[derive(Debug)]
pub struct ElseIf {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
}

#[derive(Debug)]
pub enum ParamType {
    Any,
    Number,
    String,
    Custom(String),
}

impl From<&str> for ParamType {
    fn from(s: &str) -> Self {
        match s {
            "any" => Self::Any,
            "number" => Self::Number,
            "string" => Self::String,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl From<Option<&str>> for ParamType {
    fn from(s: Option<&str>) -> Self {
        if let Some(s) = s {
            s.into()
        }
        else {
            Self::Any
        }
    }
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub kinds: Vec<ParamType>,
}

#[derive(Debug)]
pub enum Statement {
    Const {
        name: String,
        value: Arc<Value>,
    },
    Let {
        name: String,
        value: Arc<Value>,
    },
    Assign {
        identifier: String,
        value: Arc<Value>,
    },
    If{
        expr: Arc<Value>,
        block: Vec<Statement>,
        elseifs: Vec<ElseIf>,
        els: Vec<Statement>
    },
    Function {
        name: String,
        params: Vec<Param>,
        block: Vec<Statement>,
    },
    Call {
        identifier: Vec<String>,
        params: Vec<Arc<Value>>,
    },
    Class{
        name: String,
    },
    Interface{
        name: String,
    },
    Type(Type),
}