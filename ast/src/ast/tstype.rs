use super::statement::ParamType;


#[derive(Debug)]
pub struct TypeBlock {
    attributes: Vec<ParamType>,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
    pub blocks: Vec<TypeBlock>,
    pub aggregates: Vec<String>,
}