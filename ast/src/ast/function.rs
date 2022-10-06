use std::collections::HashMap;

use super::{statement::Statement, tstype::TsType, value::Value};

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub kinds: Vec<TsType>,
    pub default: Option<Value>,
}

#[derive(Debug)]
pub struct Function {
    pub name: Option<String>,
    pub template_args: HashMap<String, Vec<TsType>>,
    pub is_async: bool,
    pub params: Vec<Param>,
    pub returns: Vec<TsType>,
    pub block: Vec<Statement>,
}
