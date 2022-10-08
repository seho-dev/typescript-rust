use std::sync::Arc;

use super::{
    class::Class, function::Function, ifelse::IfElse, interface::Interface, switch::Switch,
    typedefinition::TypeDefinition, value::Value,
};

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
    If(IfElse),
    Switch(Switch),
    Return(Arc<Value>),
    Function(Function),
    Call(Arc<Value>),
    Class(Class),
    Interface(Interface),
    Type(TypeDefinition),
}
