use super::function::{Function, Param};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub attributes: Vec<Param>,
    pub methods: Vec<Function>,
}
