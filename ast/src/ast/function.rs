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
    pub params: Vec<Param>,
    pub returns: Vec<TsType>,
    pub block: Vec<Statement>,
}
