
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
            "===" => Self::Eq,
            "!==" => Self::Ne,
            _ => Self::Add,
        }
    }
}

impl From<&AssignOperation> for Operation {
    fn from(o: &AssignOperation) -> Self {
        match o {
            AssignOperation::Add => Self::Add,
            AssignOperation::Sub => Self::Sub,
            AssignOperation::Mul => Self::Mul,
            AssignOperation::Div => Self::Div,
            AssignOperation::Mod => Self::Mod,
            AssignOperation::Neutral => Self::Or,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AssignOperation {
    Neutral,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl From<&str> for AssignOperation {
    fn from(ch: &str) -> Self {
        match ch {
            "=" => Self::Neutral,
            "+=" => Self::Add,
            "-=" => Self::Sub,
            "*=" => Self::Mul,
            "/=" => Self::Div,
            "%=" => Self::Mod,
            _ => Self::Neutral,
        }
    }
}