use super::statement::Param;


#[derive(Debug)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
}

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub extends: Option<String>,
    pub attributes: Vec<Param>,
    pub methods: Vec<InterfaceMethod>,
}