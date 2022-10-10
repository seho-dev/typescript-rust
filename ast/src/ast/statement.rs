use std::sync::Arc;

use super::{
    class::Class, function::Function, ifelse::IfElse, interface::Interface, repeat::Loop,
    switch::Switch, typedefinition::TypeDefinition, value::Value, operation::AssignOperation,
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
        op: AssignOperation,
        value: Arc<Value>,
    },
    If(IfElse),
    Switch(Switch),
    Loop(Loop),
    Return(Arc<Value>),
    Function(Function),
    Call(Arc<Value>),
    Class(Class),
    Interface(Interface),
    Type(TypeDefinition),
}
