use std::sync::Arc;

use super::{statement::Statement, value::Value};

#[derive(Debug)]
pub enum Loop {
    While {
        cond: Arc<Value>,
        block: Vec<Statement>,
    },
    For {
        init: Vec<Statement>,
        cond: Arc<Value>,
        after: Arc<Value>,
        block: Vec<Statement>,
    },
    ForOf{
        name: String,
        value: Arc<Value>,
        block: Vec<Statement>,
    },
    ForIn{
        name: String,
        value: Arc<Value>,
        block: Vec<Statement>,
    },
}
