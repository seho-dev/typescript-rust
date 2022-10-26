use std::{sync::Arc, time::SystemTime};

use llvm_sys::execution_engine::{LLVMDisposeExecutionEngine, LLVMOpaqueExecutionEngine};

use super::{callbacks, context::Context, value::Value};

pub struct Module {
    id: Vec<u8>,
    pub(crate) init_fn: extern "C" fn(),
    pub(crate) ee: *mut LLVMOpaqueExecutionEngine,
    pub namespace: Arc<Context>,
}

impl Module {
    pub fn new(id: Vec<u8>) -> Self {
        Self {
            id,
            init_fn: unsafe { std::mem::transmute(0u64) },
            ee: 0 as _,
            namespace: Context::new(),
        }
    }

    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn run(&self) {
        let start = SystemTime::now();
        (self.init_fn)();
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
