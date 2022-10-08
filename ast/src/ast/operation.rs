
#[derive(Debug)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
}

impl From<&str> for Operation {
    fn from(ch: &str) -> Self {
        match ch {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            "%" => Self::Mod,
            "&&" => Self::And,
            "||" => Self::Or,
            "<" => Self::Lt,
            "<=" => Self::Lte,
            ">" => Self::Gt,
            ">=" => Self::Gte,
            "==" => Self::Eq,
            "!=" => Self::Ne,
            _ => Self::Add,
        }
    }
}
