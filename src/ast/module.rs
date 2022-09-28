use std::collections::HashMap;

use super::{value::Value, statement::Statement};

#[derive(Debug)]
pub enum ImportAlias {
    None{name: String},
    Alias{name: String, alias: String},
}

#[derive(Debug)]
pub enum Import {
    Normal{path: String},
    From{names: Vec<ImportAlias>, path: String}
}

#[derive(Debug)]
pub struct Module {
    pub exports: HashMap<String, Value>,
    pub imports: Vec<Import>,
    pub statements: Vec<Statement>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            exports: HashMap::new(),
            imports: Vec::new(),
            statements: Vec::new(),
        }
    }
}