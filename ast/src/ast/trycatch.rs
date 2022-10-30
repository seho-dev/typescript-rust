use super::statement::Statement;


#[derive(Debug)]
pub struct TryCatch {
    pub try_block: Vec<Statement>,
    pub catch_name: String,
    pub catch_block: Vec<Statement>,
}