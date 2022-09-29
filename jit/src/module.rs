use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
    fs::File,
    io::Write,
    sync::Arc, time::SystemTime,
};

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMAppendBasicBlockInContext, LLVMBuildCall2,
        LLVMBuildGlobalStringPtr, LLVMBuildRetVoid, LLVMConstReal, LLVMContextCreate,
        LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMDoubleTypeInContext, LLVMDumpModule, LLVMFloatType, LLVMFloatTypeInContext,
        LLVMFunctionType, LLVMInt64TypeInContext, LLVMModuleCreateWithNameInContext,
        LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToString, LLVMVoidTypeInContext, LLVMInt8TypeInContext, LLVMBuildGlobalString, LLVMBuildFAdd, LLVMBuildFSub,
    },
    execution_engine::{
        LLVMAddGlobalMapping, LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine,
        LLVMGetFunctionAddress, LLVMOpaqueExecutionEngine,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef},
};

use typescript_ast::ast;

use super::{callbacks, context::Context, value::Value};

pub struct ExternFunction {
    pointer: *mut c_void,
    func: LLVMValueRef,
    ft: LLVMTypeRef,
    name: CString,
}

pub struct Module {
    id: Vec<u8>,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    f64t: LLVMTypeRef,
    p64t: LLVMTypeRef,
    ee: *mut LLVMOpaqueExecutionEngine,
    extern_functions: HashMap<String, ExternFunction>,
    pub namespace: Arc<Context>,
    namespace_ptr: LLVMValueRef,
}

extern "C" fn print(val: *const Value) {
    unsafe { println!("> {:?}", *val); }
}

impl Module {
    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn from_ast(id: Vec<u8>, m: ast::Module, save_ir: Option<String>) -> Module {
        let id_hex = hex::encode(&id);
        let mut module = Self {
            id,
            builder: 0 as _,
            context: 0 as _,
            module: 0 as _,
            f64t: 0 as _,
            p64t: 0 as _,
            ee: 0 as _,
            extern_functions: HashMap::new(),
            namespace: Context::new(),
            namespace_ptr: 0 as _,
        };

        unsafe {
            module.ee = std::mem::uninitialized();
            let mut out = std::mem::zeroed();

            module.context = LLVMContextCreate();
            module.module = LLVMModuleCreateWithNameInContext(
                id_hex.as_bytes().as_ptr() as *const _,
                module.context,
            );
            module.builder = LLVMCreateBuilderInContext(module.context);
            module.f64t = LLVMDoubleTypeInContext(module.context);
            module.p64t = LLVMPointerType(LLVMInt64TypeInContext(module.context), 0);

            module.namespace_ptr = LLVMAddGlobal(
                module.module,
                LLVMInt64TypeInContext(module.context),
                b"__context\0".as_ptr() as *const _,
            );

            module.add_fn("__global_get", callbacks::global_get as *mut _, 2);
            // ctx.add_fn("__global_get_func", global_get_func as *mut _, 2);
            module.add_fn("__global_set", callbacks::global_set as *mut _, 3);
            module.add_fn("__value_delete", callbacks::value_delete as *mut _, 1);
            module.add_fn("__array_new", callbacks::array_new as *mut _, 0);
            module.add_fn("__array_push", callbacks::array_push as *mut _, 2);
            module.add_fn("__string_new", callbacks::string_new as *mut _, 0);
            module.add_fn("__add", callbacks::add as *mut _, 2);
            module.add_fn("__sub", callbacks::sub as *mut _, 2);
            module.add_fn("__mul", callbacks::mul as *mut _, 2);
            module.add_fn("__div", callbacks::div as *mut _, 2);
            {
                let mut args = Vec::new();
                args.push(LLVMPointerType(LLVMInt8TypeInContext(module.context), 0));

                module.add_fn_with("__string_from", args, callbacks::string_from as _);
            }
            {
                let mut args = Vec::new();
                args.push(module.f64t);

                module.add_fn_with("__number_new", args, callbacks::number_new as _);
            }
            module.add_fn("print", print as *mut _, 1);

            let main_func_t = LLVMFunctionType(
                LLVMVoidTypeInContext(module.context),
                std::ptr::null_mut(),
                0,
                0,
            );
            let main = LLVMAddFunction(
                module.module,
                b"__main__\0".as_ptr() as *const _,
                main_func_t,
            );

            let bb = LLVMAppendBasicBlockInContext(
                module.context,
                main,
                b"__main__entry\0".as_ptr() as *const _,
            );

            LLVMPositionBuilderAtEnd(module.builder, bb);

            // let ex = module.extern_functions.get("print").unwrap();
            // let args: Vec<LLVMValueRef> = Vec::new();
            // LLVMBuildCall2(
            //     builder,
            //     ex.ft,
            //     ex.func,
            //     args.as_ptr() as *mut LLVMValueRef,
            //     args.len() as u32,
            //     b"__print\0".as_ptr() as *const _,
            // );
            module.consume(m);

            LLVMBuildRetVoid(module.builder);

            LLVMDisposeBuilder(module.builder);

            if let Some(ir) = save_ir {
                let data = LLVMPrintModuleToString(module.module);
                let cast = CStr::from_ptr(data);
                let mut dump = File::create(ir).unwrap();
                dump.write(cast.to_bytes()).unwrap();
            }

            LLVMCreateExecutionEngineForModule(&mut module.ee, module.module, &mut out);

            module
        }
    }

    fn build_access_array(&self, parts: &Vec<LLVMValueRef>) -> LLVMValueRef {
        log::trace!(target: "typescript.build", "build_access_array >>");
        let an_ref = unsafe {
            let an = self.extern_functions.get("__array_new").unwrap();

            LLVMBuildCall2(
                self.builder,
                an.ft,
                an.func,
                0 as *mut LLVMValueRef,
                0,
                b"__array_new\0".as_ptr() as *const _,
            )
        };

        let array_push = self.extern_functions.get("__array_push").unwrap().clone();

        for p in parts {
            let args = vec![an_ref, *p];
            unsafe {
                LLVMBuildCall2(
                    self.builder,
                    array_push.ft,
                    array_push.func,
                    args.as_ptr() as *mut LLVMValueRef,
                    args.len() as u32,
                    b"__array_push\0".as_ptr() as *const _,
                )
            };
        }
        log::debug!(target: "typescript.build", "build_access_array <<");

        an_ref
    }

    fn build_global_get(&self, name: LLVMValueRef) -> LLVMValueRef {
        log::trace!(target: "typescript.build", "build_global_get >>");
        let get_global = self.extern_functions.get("__global_get").unwrap();
        let value_delete = self.extern_functions.get("__value_delete").unwrap();
        let args = vec![self.namespace_ptr, name];
        let delete_args = vec![name];

        unsafe {
            let func = LLVMBuildCall2(
                self.builder,
                get_global.ft,
                get_global.func,
                args.as_ptr() as *mut _,
                args.len() as u32,
                b"global_get\0".as_ptr() as *const _,
            );

            LLVMBuildCall2(
                self.builder,
                value_delete.ft,
                value_delete.func,
                delete_args.as_ptr() as *mut LLVMValueRef,
                delete_args.len() as u32,
                b"__value_delete\0".as_ptr() as *const _,
            );
            log::trace!(target: "typescript.build", "build_global_get <<");

            func
        }
    }

    fn build_global_set(&self, name: LLVMValueRef, value: LLVMValueRef) -> LLVMValueRef {
        log::trace!(target: "typescript.build", "build_global_set >>");

        let global_set = self.extern_functions.get("__global_set").unwrap();
        let value_delete = self.extern_functions.get("__value_delete").unwrap();
        let args = vec![self.namespace_ptr, name, value];
        let delete_args = vec![name];

        unsafe {
            let ret = LLVMBuildCall2(
                self.builder,
                global_set.ft,
                global_set.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__global_set\0".as_ptr() as *const _,
            );

            LLVMBuildCall2(
                self.builder,
                value_delete.ft,
                value_delete.func,
                delete_args.as_ptr() as *mut LLVMValueRef,
                delete_args.len() as u32,
                b"__value_delete\0".as_ptr() as *const _,
            );
            log::trace!(target: "typescript.build", "build_global_set <<");

            ret
        }
    }

    fn build_string(&self, s: &str) -> LLVMValueRef {
        log::debug!(target: "typescript.build", "build_string");

        unsafe {
            // let cstr = LLVMBuildGlobalStringPtr(
            //     self.builder,
            //     CString::new(s).unwrap().as_ptr(),
            //     b"__str\0".as_ptr() as *const _,
            // );
            let cs = CString::new(s).unwrap();
            let cstr = LLVMBuildGlobalString(
                self.builder,
                cs.as_ptr(),
                b"__str\0".as_ptr() as *const _,
            );

            let args = vec![cstr];
            let string_from = self.extern_functions.get("__string_from").unwrap();
            LLVMBuildCall2(
                self.builder,
                string_from.ft,
                string_from.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__string_from\0".as_ptr() as *const _,
            )
        }
    }

    fn build_op(&self, op: &ast::operation::Operation, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        unsafe {
            match op {
                ast::operation::Operation::Add => {
                    LLVMBuildFAdd(
                        self.builder, 
                        left, 
                        right, 
                        b"__add\0".as_ptr() as _
                    )
                }
                ast::operation::Operation::Sub => {
                    LLVMBuildFSub(
                        self.builder, 
                        left, 
                        right, 
                        b"__sub\0".as_ptr() as _
                    )
                }
                _ => { 0 as _ }
            }
        }
    }

    fn build_generic_op(&self, op: &ast::operation::Operation, left_ref: LLVMValueRef, right_ref: LLVMValueRef) -> LLVMValueRef {
        use ast::operation::Operation;

        log::debug!(target: "typescript.build", "generic_op");

        let call = match op {
            Operation::Add => self.extern_functions.get("__add").unwrap(),
            Operation::Sub => self.extern_functions.get("__sub").unwrap(),
            Operation::Mul => self.extern_functions.get("__mul").unwrap(),
            Operation::Div => self.extern_functions.get("__div").unwrap(),
            Operation::Eq => self.extern_functions.get("__add").unwrap(),
            Operation::Ne => self.extern_functions.get("__add").unwrap(),
            Operation::Gt => self.extern_functions.get("__add").unwrap(),
            Operation::Gte => self.extern_functions.get("__add").unwrap(),
            Operation::Lt => self.extern_functions.get("__add").unwrap(),
            Operation::Lte => self.extern_functions.get("__add").unwrap(),
            Operation::Mod => self.extern_functions.get("__add").unwrap(),
            Operation::And => self.extern_functions.get("__add").unwrap(),
            Operation::Or => self.extern_functions.get("__add").unwrap(),
        };
    
        let args = vec![left_ref, right_ref];
        unsafe {
            LLVMBuildCall2(
                self.builder,
                call.ft,
                call.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__op_res\0".as_ptr() as *const _,
            )
        }
    }

    fn build_value(&self, value: Arc<ast::value::Value>) -> LLVMValueRef {
        unsafe {
            match &*value {
                ast::value::Value::Number(n) => {
                    let float_new = self.extern_functions.get("__number_new").unwrap();
                    let args = vec![LLVMConstReal(self.f64t, *n)];
                    LLVMBuildCall2(
                        self.builder,
                        float_new.ft,
                        float_new.func,
                        args.as_ptr() as *mut LLVMValueRef,
                        args.len() as u32,
                        b"__number_new\0".as_ptr() as *const _,
                    )
                }
                ast::value::Value::String(n) => self.build_string(&n),
                ast::value::Value::Expression { left, op, right } => {
                    let left_ref = self.build_value(left.clone());
                    let right_ref = self.build_value(right.clone());
                    // self.build_op(op, left_ref, right_ref)
                    self.build_generic_op(op, left_ref, right_ref)
                }
                ast::value::Value::Identifier(n) => {
                    let parts = n.iter().map(|s| self.build_string(s)).collect();
                    let access = self.build_access_array(&parts);
                    self.build_global_get(access)
                }
                _ => 0 as _,
            }
        }
    }

    fn consume_statement(&self, statement: ast::statement::Statement) -> LLVMValueRef {
        match statement {
            ast::statement::Statement::Const { name, value } => {
                let name_ref = self.build_string(&name);
                let value_ref = self.build_value(value);
                self.build_global_set(name_ref, value_ref)
            }
            ast::statement::Statement::Let { name, value } => {
                let name_ref = self.build_string(&name);
                let value_ref = self.build_value(value);
                self.build_global_set(name_ref, value_ref)
            }
            ast::statement::Statement::Call { identifier, params } => {
                if identifier.len() == 1 {
                    if let Some(n) = self.extern_functions.get(&identifier[0]) {
                        let mut args: Vec<LLVMValueRef> = Vec::new();

                        for p in params {
                            args.push(self.build_value(p));
                        }

                        unsafe {
                            return LLVMBuildCall2(
                                self.builder,
                                n.ft,
                                n.func,
                                args.as_ptr() as _,
                                args.len() as _,
                                b"__call\0".as_ptr() as _
                            );
                        }
                    }
                }
                0 as _
            }
            ast::statement::Statement::Function { name, params, block } => {
                0 as _
            }
            _ => 0 as _,
        }
    }

    fn consume(&self, module: ast::Module) {
        for stmnt in module.statements {
            self.consume_statement(stmnt);
        }
    }

    pub fn add_fn(&mut self, name: &str, f: *mut c_void, cnt: u32) {
        let mut args = Vec::new();
        for _ in 0..cnt {
            args.push(self.p64t);
        }

        self.add_fn_with(name, args, f);
    }

    pub fn add_fn_with(&mut self, name: &str, args: Vec<LLVMTypeRef>, f: *mut c_void) {
        let cname = CString::new(name).unwrap();
        let (func, ft) = unsafe {
            let ft = LLVMFunctionType(self.p64t, args.as_ptr() as *mut _, args.len() as u32, 0);
            (LLVMAddFunction(self.module, cname.as_ptr(), ft), ft)
        };

        let ex = ExternFunction {
            pointer: f,
            func,
            ft,
            name: cname,
        };
        self.extern_functions.insert(name.to_string(), ex);
    }

    pub fn run(&self) {
        unsafe {
            #[cfg(feature = "trace")]
            log::debug!(target: "typescript.jit", "add gloabl mapping");

            let ns_ptr = Arc::into_raw(self.namespace.clone());
            LLVMAddGlobalMapping(self.ee, self.namespace_ptr, ns_ptr as *mut _);

            for ex in self.extern_functions.values() {
                LLVMAddGlobalMapping(self.ee, ex.func, ex.pointer);

                // self.runtime_variables.insert(
                //     CString::new(name.as_bytes()).unwrap(),
                //     Rc::new(Value::Lambda(func as usize))
                // );
            }

            let addr = LLVMGetFunctionAddress(self.ee, b"__main__\0".as_ptr() as *const _);

            let start = SystemTime::now();
            let f: extern "C" fn() = std::mem::transmute(addr);

            f();
            let dur = start.elapsed().unwrap();
            log::info!(target: "typescript.module", "main: {}.{:06}", dur.as_secs(), dur.subsec_micros());
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.ee);
            LLVMContextDispose(self.context);
        }
    }
}
