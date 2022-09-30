use std::sync::Arc;

use super::operation::Operation;


#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Identifier(Vec<String>),
    Boolean(bool),
    Null,
    Undefined,
    Expression{left: Arc<Value>, op: Operation, right: Arc<Value>},
}