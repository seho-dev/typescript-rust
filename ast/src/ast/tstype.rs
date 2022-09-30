use super::statement::Param;



#[derive(Debug)]
pub struct TypeBlock {
    pub attributes: Vec<Param>,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
    pub blocks: Vec<TypeBlock>,
    pub aggregates: Vec<String>,
}