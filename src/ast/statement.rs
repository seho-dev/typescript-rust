use std::sync::Arc;

use super::value::Value;

#[derive(Debug)]
pub struct ElseIf {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
}

#[derive(Debug)]
pub enum ParamType {
    Any,
    Number,
}

impl From<Option<&str>> for ParamType {
    fn from(s: Option<&str>) -> Self {
        if let Some(s) = s {
            match s {
                "number" => Self::Number,
                _ => Self::Any,
            }
        }
        else {
            Self::Any
        }
    }
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub kind: ParamType,
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
    Class,
    Interface,
}