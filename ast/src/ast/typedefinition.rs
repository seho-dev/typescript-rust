use super::function::Param;


#[derive(Debug)]
pub struct TypeBlock {
    pub attributes: Vec<Param>,
}

#[derive(Debug)]
pub struct TypeDefinition {
    pub name: String,
    pub blocks: Vec<TypeBlock>,
    pub aggregates: Vec<String>,
}