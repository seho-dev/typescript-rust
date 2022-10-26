use std::{collections::HashMap, ffi::{CStr, CString, c_void}, fs::File, io::Write, sync::Arc, time::SystemTime};

use llvm_sys::{
    analysis::{LLVMVerifyModule, LLVMVerifierFailureAction},
    core::{
        LLVMAddGlobal, LLVMBuildRetVoid, LLVMDisposeBuilder, LLVMDisposeMessage,
        LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMModuleCreateWithNameInContext,
        LLVMPointerType, LLVMPrintModuleToString, LLVMPositionBuilderAtEnd, LLVMAppendBasicBlockInContext, LLVMAddFunction, LLVMFunctionType, LLVMVoidTypeInContext, LLVMDoubleTypeInContext, LLVMCreateBuilderInContext, LLVMContextCreate,
    },
    execution_engine::{
        LLVMAddGlobalMapping, LLVMCreateExecutionEngineForModule, LLVMGetFunctionAddress,
    },
    prelude::{
        LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
    },
};
use typescript_ast::ast;

use crate::{error::JitError, stdlib::Array, Module, Value, callbacks};

use self::statement::build_statement;

mod functions;
mod ifs;
mod repeat;
mod statement;
mod switch;
mod value;

extern "C" fn print(val: *const Value) {
    unsafe {
        println!("> {:?}", *val);
    }
}

pub struct ExternFunction {
    pointer: *mut c_void,
    func: LLVMValueRef,
    ft: LLVMTypeRef,
    name: CString,
}

pub struct InternFunction {
    func: LLVMValueRef,
    ft: LLVMTypeRef,
    name: CString,
}

pub struct Builder {
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    current_function: LLVMValueRef,
    current_block: LLVMBasicBlockRef,
    f64t: LLVMTypeRef,
    p64t: LLVMTypeRef,
    namespace_ptr: LLVMValueRef,
    string_cache: HashMap<String, LLVMValueRef>,
    extern_functions: HashMap<String, ExternFunction>,
    function_cache: HashMap<String, InternFunction>,
    id: Option<Vec<u8>>,
    stdlib: bool,
    save_ir: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            builder: 0 as _,
            context: 0 as _,
            module: 0 as _,
            current_function: 0 as _,
            current_block: 0 as _,
            f64t: 0 as _,
            p64t: 0 as _,
            namespace_ptr: 0 as _,
            string_cache: HashMap::new(),
            extern_functions: HashMap::new(),
            function_cache: HashMap::new(),
            id: None,
            stdlib: false,
            save_ir: None,
        }
    }

    pub fn id(&mut self, id: Vec<u8>) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn save_ir(&mut self, filename: &str) -> &mut Self {
        self.save_ir = Some(filename.into());
        self
    }

    pub fn standard_library(&mut self) -> &mut Self {
        self.stdlib = true;
        self
    }

    pub fn add_fn(&mut self, name: &str, f: *mut c_void, cnt: u32) {
        let mut args = Vec::new();
        for _ in 0..cnt {
            args.push(self.p64t);
        }

        self.add_fn_with(name, self.p64t, args, f);
    }

    pub fn add_fn_with(
        &mut self,
        name: &str,
        ret: LLVMTypeRef,
        args: Vec<LLVMTypeRef>,
        f: *mut c_void,
    ) {
        let cname = CString::new(name).unwrap();
        let (func, ft) = unsafe {
            let ft = LLVMFunctionType(ret, args.as_ptr() as *mut _, args.len() as u32, 0);
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

    fn init(&mut self) {
        let id_hex = if let Some(id) = &self.id {
            hex::encode(&id)
        } else {
            "noname".to_owned()
        };

        unsafe {
            self.context = LLVMContextCreate();
            self.module = LLVMModuleCreateWithNameInContext(
                id_hex.as_bytes().as_ptr() as *const _,
                self.context,
            );
            self.builder = LLVMCreateBuilderInContext(self.context);
            self.f64t = LLVMDoubleTypeInContext(self.context);
            self.p64t = LLVMPointerType(LLVMInt64TypeInContext(self.context), 0);

            self.namespace_ptr = LLVMAddGlobal(
                self.module,
                LLVMInt64TypeInContext(self.context),
                b"__context\0".as_ptr() as *const _,
            );

            self.add_fn("__global_null", callbacks::global_null as *mut _, 0);
            self.add_fn("__global_get", callbacks::global_get as *mut _, 2);
            self.add_fn("__global_set", callbacks::global_set as *mut _, 3);
            self.add_fn("__get_attr", callbacks::get_attr as *mut _, 2);
            self.add_fn("__value_delete", callbacks::value_delete as *mut _, 1);
            self.add_fn("__string_new", callbacks::string_new as *mut _, 0);
            self.add_fn("__string_copy", callbacks::string_copy as *mut _, 1);
            self.add_fn("__add", callbacks::add as *mut _, 2);
            self.add_fn("__sub", callbacks::sub as *mut _, 2);
            self.add_fn("__mul", callbacks::mul as *mut _, 2);
            self.add_fn("__div", callbacks::div as *mut _, 2);
            self.add_fn("__mod", callbacks::_mod as *mut _, 2);
            self.add_fn("__gt", callbacks::gt as *mut _, 2);
            self.add_fn("__gte", callbacks::gte as *mut _, 2);
            self.add_fn("__lt", callbacks::lt as *mut _, 2);
            self.add_fn("__lte", callbacks::lte as *mut _, 2);
            self.add_fn("__eq", callbacks::eq as *mut _, 2);
            self.add_fn("__neq", callbacks::neq as *mut _, 2);
            self.add_fn("__and", callbacks::and as *mut _, 2);
            self.add_fn("__or", callbacks::or as *mut _, 2);

            {
                let mut args = Vec::new();
                args.push(self.p64t);
                let ret = LLVMInt64TypeInContext(self.context);

                self.add_fn_with("__get_func_addr", ret, args, callbacks::get_func_addr as _);
            }
            {
                let mut args = Vec::new();
                args.push(self.p64t);
                let ret = LLVMInt8TypeInContext(self.context);

                self.add_fn_with("__to_bool", ret, args, callbacks::to_bool as _);
            }
            {
                let mut args = Vec::new();
                args.push(LLVMPointerType(LLVMInt8TypeInContext(self.context), 0));

                self.add_fn_with(
                    "__string_from",
                    self.p64t,
                    args,
                    callbacks::string_from as _,
                );
            }
            {
                let mut args = Vec::new();
                args.push(self.f64t);

                self.add_fn_with("__number_new", self.p64t, args, callbacks::number_new as _);
            }

            if self.stdlib {
                Array::register(self);

                self.add_fn("print", print as *mut _, 1);
            }
        }
    }

    fn build_main(&mut self, module: &ast::Module) -> Result<(), JitError> {
        let start = SystemTime::now();
        unsafe {
            let main_func_t = LLVMFunctionType(
                LLVMVoidTypeInContext(self.context),
                std::ptr::null_mut(),
                0,
                0,
            );
            let main =
                LLVMAddFunction(self.module, b"__main__\0".as_ptr() as *const _, main_func_t);
            self.current_function = main;

            let bb = LLVMAppendBasicBlockInContext(
                self.context,
                main,
                b"__main__entry\0".as_ptr() as *const _,
            );

            LLVMPositionBuilderAtEnd(self.builder, bb);
            self.current_block = bb;
        }

        for stmnt in &module.statements {
            build_statement(self, stmnt);
        }

        unsafe {
            LLVMBuildRetVoid(self.builder);

            {
                let mut data = 0u8 as _;
                let ret = LLVMVerifyModule(
                    self.module,
                    LLVMVerifierFailureAction::LLVMReturnStatusAction,
                    &mut data,
                );

                if ret != 0 {
                    let cast = CStr::from_ptr(data);
                    let error = JitError::ModuleVerify(cast.to_str().unwrap().into());
                    LLVMDisposeMessage(data);
                    return Err(error.into());
                }
            }
        }

        let dur = start.elapsed().unwrap();
        log::info!(
            "ast consumtion: {}.{:06}",
            dur.as_secs(),
            dur.subsec_micros()
        );

        Ok(())
    }

    unsafe fn create_mapping(&mut self, module: &mut Module) {
        let start = SystemTime::now();
        let ns_ptr = Arc::into_raw(module.namespace.clone());
        LLVMAddGlobalMapping(module.ee, self.namespace_ptr, ns_ptr as *mut _);

        for ex in self.extern_functions.values() {
            LLVMAddGlobalMapping(module.ee, ex.func, ex.pointer);

            // self.runtime_variables.insert(
            //     CString::new(name.as_bytes()).unwrap(),
            //     Rc::new(Value::Lambda(func as usize))
            // );
        }
        let dur = start.elapsed().unwrap();
        log::info!(
            "create mapping: {}.{:06}",
            dur.as_secs(),
            dur.subsec_micros()
        );

        let start = SystemTime::now();
        // JIT compiliation is defered until needed. So the real compilation starts here.
        let addr = LLVMGetFunctionAddress(module.ee, b"__main__\0".as_ptr() as *const _);
        let dur = start.elapsed().unwrap();
        log::info!("llvm compilation: {}.{:06}", dur.as_secs(), dur.subsec_micros());

        module.init_fn = std::mem::transmute(addr);
    }

    pub fn build(&mut self, module: &ast::Module) -> Result<Arc<Module>, JitError> {
        log::info!("init");
        self.init();
        log::info!("main");
        self.build_main(module);

        log::info!("new module");
        let mut module = Module::new(self.id.clone().unwrap_or(Vec::new()));

        unsafe {
            if let Some(ir) = &self.save_ir {
                let data = LLVMPrintModuleToString(self.module);
                let cast = CStr::from_ptr(data);
                let mut dump = File::create(ir).unwrap();
                dump.write(cast.to_bytes()).unwrap();
            }

            module.ee = std::mem::zeroed();
            let mut out = std::mem::zeroed();

            LLVMCreateExecutionEngineForModule(&mut module.ee, self.module, &mut out);

            self.create_mapping(&mut module);
        }

        Ok(Arc::new(module))
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}
