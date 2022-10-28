use std::{sync::Arc, time::SystemTime};

use llvm_sys::execution_engine::{LLVMDisposeExecutionEngine, LLVMExecutionEngineRef};

use super::context::Context;

pub struct Module {
    id: Vec<u8>,
    pub(crate) init_fn: Option<extern "C" fn()>,
    pub(crate) ee: LLVMExecutionEngineRef,
    pub namespace: Arc<Context>,
}

impl Module {
    pub fn new(id: Vec<u8>) -> Self {
        Self {
            id,
            init_fn: None,
            ee: 0 as _,
            namespace: Context::new(),
        }
    }

    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn run(&self) {
        let start = SystemTime::now();
        if let Some(func) = self.init_fn {
            (func)();
        }
        let dur = start.elapsed().unwrap();
        log::info!("main: {}.{:06}", dur.as_secs(), dur.subsec_micros());
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            if self.ee != 0 as _ {
                LLVMDisposeExecutionEngine(self.ee);
            }
        }
    }
}
