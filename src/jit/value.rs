use std::{sync::Arc, collections::HashMap};


#[derive(Debug)]
pub enum Value {
    Number(f64),
    Str(String),
    Array(Vec<Arc<Value>>),
    Object(HashMap<String, Arc<Value>>),
    Function,
    Null,
}

impl Drop for Value {
    fn drop(&mut self) {
        #[cfg(feature = "trace")]
        log::debug!(target: "typescript.value", "value dropped: {:?}", self);
    }
}