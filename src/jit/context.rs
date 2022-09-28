use std::{collections::HashMap, sync::Arc};

use super::value::Value;

pub struct Context {
    pub parent: Option<Arc<Context>>,
    pub variables: HashMap<String, Arc<Value>>,
}

impl Context {
    pub fn new() -> Arc<Context> {
        Arc::new(Self {
            parent: None,
            variables: HashMap::new(),
        })
    }
}