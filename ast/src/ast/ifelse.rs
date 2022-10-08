use std::sync::Arc;

use super::{value::Value, statement::Statement};

#[derive(Debug)]
pub struct ElseIf {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
}

#[derive(Debug)]
pub struct IfElse {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
    pub elseifs: Vec<ElseIf>,
    pub els: Vec<Statement>,
}