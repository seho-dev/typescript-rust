use super::statement::Param;


#[derive(Debug)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<Param>,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub attributes: Vec<Param>,
    pub methods: Vec<ClassMethod>,
}