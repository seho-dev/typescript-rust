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
        after: Vec<Statement>,
        block: Vec<Statement>,
    },
    ForOf,
    ForIn,
}
