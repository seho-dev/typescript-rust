
#[derive(Debug)]
pub enum TsType {
    Any,
    Number,
    String,
    Boolean,
    Null,
    Custom(String),
}

impl From<&str> for TsType {
    fn from(s: &str) -> Self {
        match s {
            "any" => Self::Any,
            "number" => Self::Number,
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "null" => Self::Null,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl From<Option<&str>> for TsType {
    fn from(s: Option<&str>) -> Self {
        if let Some(s) = s {
            s.into()
        } else {
            Self::Any
        }
    }
}