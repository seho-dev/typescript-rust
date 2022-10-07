use std::sync::Arc;

use super::{value::Value, statement::Statement};


#[derive(Debug)]
pub struct Case {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
}

#[derive(Debug)]
pub struct Switch {
    pub value:  Arc<Value>,
    pub branches: Vec<Case>,
    pub default: Option<Vec<Statement>>,
}