use std::sync::Arc;

use super::operation::{AssignOperation, Operation};

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Identifier(Vec<String>),
    Boolean(bool),
    Array(Vec<Arc<Value>>),
    Call {
        identifier: Vec<String>,
        args: Vec<Arc<Value>>,
    },
    Null,
    Undefined,
    Expression {
        left: Arc<Value>,
        op: Operation,
        right: Arc<Value>,
    },
    Assign {
        identifier: String,
        op: AssignOperation,
        value: Arc<Value>,
    },
}
