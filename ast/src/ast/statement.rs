use std::sync::Arc;

use super::{
    class::Class, function::Function, interface::Interface, typedefinition::TypeDefinition,
    value::Value,
};

#[derive(Debug)]
pub struct ElseIf {
    pub expr: Arc<Value>,
    pub block: Vec<Statement>,
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
    If {
        expr: Arc<Value>,
        block: Vec<Statement>,
        elseifs: Vec<ElseIf>,
        els: Vec<Statement>,
    },
    Return(Arc<Value>),
    Function(Function),
    Call(Arc<Value>),
    Class(Class),
    Interface(Interface),
    Type(TypeDefinition),
}
