use super::function::{Function, Param};

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub extends: Option<String>,
    pub attributes: Vec<Param>,
    pub methods: Vec<Function>,
}
