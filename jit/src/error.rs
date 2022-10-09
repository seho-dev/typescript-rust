use std::{error::Error, fmt};


#[derive(Debug)]
pub enum JitError {
    ModuleVerify(String),
}

impl Error for JitError {}

impl fmt::Display for JitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
