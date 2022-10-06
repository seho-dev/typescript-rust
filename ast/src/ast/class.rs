use std::collections::HashMap;

use super::{
    function::{Function, Param},
    tstype::TsType,
};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub attributes: Vec<Param>,
    pub methods: Vec<Function>,
    pub template_args: HashMap<String, Vec<TsType>>,
}
